//! Schema and query-shape behaviors that caused silent production bugs in a proving repository
//! (Astra, 2026-09-28..10-01). Each test pins the 3.2.4 behavior the bug depended on.

use surrealdb::{
    engine::local::{Db, Mem},
    types::Value,
    Surreal,
};

async fn memory_db() -> surrealdb::Result<Surreal<Db>> {
    let db = Surreal::new::<Mem>(()).await?;
    db.use_ns("repro").use_db("repro").await?;
    Ok(db)
}

/// A field written by code but never defined is refused by a SCHEMAFULL table. A test that
/// creates its rows in a table nobody defined gets a schemaless table, so the same write passes
/// there: the fixture hides the production failure.
#[tokio::test]
async fn schemafull_refuses_an_undefined_top_level_field_an_undefined_table_hides_it(
) -> surrealdb::Result<()> {
    let db = memory_db().await?;
    db.query(
        "DEFINE TABLE user SCHEMAFULL;
         DEFINE FIELD telegram_id ON TABLE user TYPE int;
         CREATE user:one SET telegram_id = 1;",
    )
    .await?
    .check()?;

    let refused = db
        .query("UPDATE user:one SET language_preference = 'en'")
        .await?;
    assert!(
        refused.check().is_err(),
        "an undefined top-level field on a SCHEMAFULL table must be refused"
    );

    db.query(
        "CREATE fixture_user:one SET telegram_id = 1;
         UPDATE fixture_user:one SET language_preference = 'en';",
    )
    .await?
    .check()?;
    Ok(())
}

/// `ORDER BY` a field the projection does not select is a parse error. It fails the whole
/// request, so a valid statement earlier in the same request never runs either.
#[tokio::test]
async fn order_by_needs_its_field_selected_and_the_parse_error_fails_the_whole_request(
) -> surrealdb::Result<()> {
    let db = memory_db().await?;
    db.query("CREATE log:one SET text = 'a', created_at = time::now()")
        .await?
        .check()?;

    let result = db
        .query(
            "CREATE log:two SET text = 'b', created_at = time::now();
             SELECT text FROM log ORDER BY created_at DESC LIMIT 1;",
        )
        .await;
    let failed = match result {
        Err(_) => true,
        Ok(response) => response.check().is_err(),
    };
    assert!(failed, "ORDER BY on an unselected field must fail");

    let mut response = db.query("SELECT VALUE record::id(id) FROM log").await?;
    let ids: Vec<String> = response.take(0)?;
    assert_eq!(ids, vec!["one".to_string()], "the CREATE in the failed request never ran");

    db.query("SELECT text, created_at FROM log ORDER BY created_at DESC LIMIT 1")
        .await?
        .check()?;
    Ok(())
}

/// JSON has no datetime type. A timestamp written through `serde_json` (for example
/// `serde_json::to_value` of a `chrono::DateTime`) is stored as a string, and a string never
/// matches a datetime comparison: the row silently drops out of the result. Casting finds it,
/// and a `TYPE datetime` field turns the silent string into a refused write.
#[tokio::test]
async fn json_timestamps_are_strings_and_drop_out_of_datetime_comparisons(
) -> surrealdb::Result<()> {
    let db = memory_db().await?;
    let row = serde_json::json!({ "created_at": "2026-09-30T15:41:11Z" });
    let _: Option<Value> = db.create(("log", "json")).content(row).await?;
    db.query("CREATE log:native SET created_at = d'2026-09-30T15:41:11Z'")
        .await?
        .check()?;

    let mut response = db
        .query("SELECT VALUE type::is_string(created_at) FROM log:json")
        .await?;
    let is_text: Vec<bool> = response.take(0)?;
    assert_eq!(is_text, vec![true], "the JSON timestamp is stored as text");

    let mut response = db
        .query("SELECT VALUE record::id(id) FROM log WHERE created_at > d'2026-09-30T00:00:00Z'")
        .await?;
    let matched: Vec<String> = response.take(0)?;
    assert_eq!(matched, vec!["native".to_string()], "the text timestamp is silently skipped");

    let mut response = db
        .query(
            "SELECT VALUE record::id(id) FROM log \
             WHERE <datetime> created_at > d'2026-09-30T00:00:00Z'",
        )
        .await?;
    let mut cast: Vec<String> = response.take(0)?;
    cast.sort();
    assert_eq!(cast, vec!["json".to_string(), "native".to_string()]);

    db.query(
        "DEFINE TABLE typed_log SCHEMALESS;
         DEFINE FIELD created_at ON TABLE typed_log TYPE datetime;",
    )
    .await?
    .check()?;
    let typed: surrealdb::Result<Option<Value>> = db
        .create(("typed_log", "json"))
        .content(serde_json::json!({ "created_at": "2026-09-30T15:41:11Z" }))
        .await;
    assert!(
        typed.is_err(),
        "a TYPE datetime field refuses the JSON text instead of storing it"
    );
    Ok(())
}

/// `UPSERT … MERGE` keeps fields the payload does not mention; `UPSERT … CONTENT` replaces the
/// whole record, so a partial reading written with CONTENT erases what an earlier one stored.
#[tokio::test]
async fn upsert_merge_keeps_absent_fields_while_content_replaces_the_record(
) -> surrealdb::Result<()> {
    let db = memory_db().await?;
    db.query(
        "UPSERT shipment:merged CONTENT { service: 'SEDEX', total: 5170 };
         UPSERT shipment:merged MERGE { verified: true };
         UPSERT shipment:replaced CONTENT { service: 'SEDEX', total: 5170 };
         UPSERT shipment:replaced CONTENT { verified: true };",
    )
    .await?
    .check()?;

    let mut response = db
        .query(
            "SELECT VALUE service FROM shipment:merged;
             SELECT VALUE service FROM shipment:replaced;",
        )
        .await?;
    let merged: Vec<Option<String>> = response.take(0)?;
    let replaced: Vec<Option<String>> = response.take(1)?;
    assert_eq!(merged, vec![Some("SEDEX".to_string())]);
    assert_eq!(replaced, vec![None]);
    Ok(())
}

/// A SCHEMALESS table still enforces the fields it defines (types and ASSERTs) while accepting
/// any other field. An optional enumeration is `TYPE option<string>` with an ASSERT that allows
/// NONE.
#[tokio::test]
async fn schemaless_tables_still_enforce_the_fields_they_define() -> surrealdb::Result<()> {
    let db = memory_db().await?;
    db.query(
        "DEFINE TABLE shipment SCHEMALESS;
         DEFINE FIELD carrier ON TABLE shipment TYPE string ASSERT $value IN ['correios'];
         DEFINE FIELD language ON TABLE shipment TYPE option<string>
             ASSERT $value = NONE OR $value IN ['pt-br', 'en'];
         CREATE shipment:free SET carrier = 'correios', anything_else = 1;
         CREATE shipment:no_language SET carrier = 'correios';
         CREATE shipment:english SET carrier = 'correios', language = 'en';",
    )
    .await?
    .check()?;

    let carrier = db
        .query("CREATE shipment:other SET carrier = 'jadlog'")
        .await?;
    assert!(carrier.check().is_err(), "an unlisted carrier must be refused");

    let language = db
        .query("CREATE shipment:french SET carrier = 'correios', language = 'fr'")
        .await?;
    assert!(language.check().is_err(), "an unlisted language must be refused");
    Ok(())
}
