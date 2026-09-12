use surrealdb::{
    engine::local::{Db, Mem, SurrealKv},
    types::{RecordId, SurrealValue, Value},
    Surreal,
};

#[derive(Debug, SurrealValue)]
struct NativeRow {
    id: RecordId,
    name: String,
}

#[derive(Debug, SurrealValue)]
struct WrongRow {
    id: String,
    name: String,
}

async fn memory_db() -> surrealdb::Result<Surreal<Db>> {
    let db = Surreal::new::<Mem>(()).await?;
    db.use_ns("repro").use_db("repro").await?;
    Ok(db)
}

#[tokio::test]
async fn intrinsic_id_is_record_id_not_string() -> surrealdb::Result<()> {
    let db = memory_db().await?;
    db.query("CREATE person:one SET name = 'Alice'")
        .await?
        .check()?;

    let mut response = db.query("SELECT * FROM person:one").await?;
    let row: Option<NativeRow> = response.take(0)?;
    let row = row.expect("person:one should exist");
    assert_eq!(row.name, "Alice");
    assert_eq!(row.id.table.to_string(), "person");

    let mut response = db.query("SELECT * FROM person:one").await?;
    let wrong = response.take::<Option<WrongRow>>(0);
    assert!(wrong.is_err(), "intrinsic record id must not decode as String");
    Ok(())
}

#[tokio::test]
async fn outer_query_success_can_contain_statement_errors() -> surrealdb::Result<()> {
    let db = memory_db().await?;
    let mut response = db
        .query(
            "DEFINE TABLE user SCHEMAFULL;
             DEFINE FIELD name ON TABLE user TYPE string;
             CREATE user:good SET name = 'Alice';
             CREATE user:bad SET name = 42;",
        )
        .await?;

    let errors = response.take_errors();
    assert!(
        !errors.is_empty(),
        "outer query returned Ok, but the invalid typed statement must be reported"
    );
    Ok(())
}

#[tokio::test]
async fn schemafull_nested_fields_are_strict_unless_flexible() -> surrealdb::Result<()> {
    let db = memory_db().await?;

    db.query(
        "DEFINE TABLE strict SCHEMAFULL;
         DEFINE FIELD metadata ON TABLE strict TYPE object;",
    )
    .await?
    .check()?;

    let mut strict = db
        .query("CREATE strict:one SET metadata = { extra: 'blocked' }")
        .await?;
    assert!(
        !strict.take_errors().is_empty(),
        "undeclared nested field should fail on SCHEMAFULL object"
    );

    db.query(
        "DEFINE TABLE flexible SCHEMAFULL;
         DEFINE FIELD metadata ON TABLE flexible TYPE object FLEXIBLE;
         CREATE flexible:one SET metadata = { extra: 'allowed' };",
    )
    .await?
    .check()?;

    Ok(())
}

#[tokio::test]
async fn surrealkv_engine_selector_produces_local_db_handle() -> surrealdb::Result<()> {
    let dir = tempfile::tempdir().expect("temporary directory");
    let db_path = dir.path().join("surrealkv");
    let db_path = db_path.to_str().expect("utf-8 temp path");

    let db: Surreal<Db> = Surreal::new::<SurrealKv>(db_path).await?;
    db.use_ns("repro").use_db("repro").await?;

    let mut response = db.query("RETURN 1").await?;
    let one: Option<i64> = response.take(0)?;
    assert_eq!(one, Some(1));
    Ok(())
}

#[tokio::test]
async fn explicit_transaction_failure_rolls_back_prior_write() -> surrealdb::Result<()> {
    let db = memory_db().await?;

    // Keep schema existence outside the transaction so the post-rollback
    // assertion tests row atomicity rather than whether DDL was also rolled back.
    db.query("DEFINE TABLE tx_item SCHEMALESS")
        .await?
        .check()?;

    let response = db
        .query(
            "BEGIN TRANSACTION;
             CREATE tx_item:one SET value = 1;
             THROW 'forced rollback';
             COMMIT TRANSACTION;",
        )
        .await?;
    assert!(response.check().is_err(), "forced transaction failure must surface");

    let mut response = db.query("SELECT * FROM tx_item").await?;
    let rows: Vec<Value> = response.take(0)?;
    assert!(rows.is_empty(), "rolled-back write must not survive");
    Ok(())
}
