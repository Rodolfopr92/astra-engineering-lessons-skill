//! Pitfalls met while integrating SurrealQL and Rust SDK code that had been written without ever
//! being compiled or executed (DLF Block 04, 2026-10-29). Each test pins the 3.2.4 behavior that
//! a repair depended on. Engine: in-memory `Mem`; the proving repository ran embedded SurrealKV.
//!
//! The query response type of SDK 3.2.4 is `surrealdb::IndexedResults`; the 2.x name
//! `surrealdb::Response` no longer exists, which this file's imports also pin.

use std::collections::HashMap;

use surrealdb::{
    engine::local::{Db, Mem},
    types::{RecordId, SurrealValue, ToSql},
    IndexedResults, Surreal,
};

async fn memory_db() -> Result<Surreal<Db>, String> {
    let db = Surreal::new::<Mem>(()).await.map_err(|e| e.to_string())?;
    db.use_ns("repro")
        .use_db("repro")
        .await
        .map_err(|e| e.to_string())?;
    Ok(db)
}

/// Runs one request and folds both failure paths (the outer `Err` of `.await`, and a statement
/// error surfaced by `.check()`) into the error text.
async fn run(db: &Surreal<Db>, sql: &str) -> Result<IndexedResults, String> {
    match db.query(sql).await {
        Ok(response) => response.check().map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

fn error_text(result: Result<IndexedResults, String>) -> String {
    match result {
        Ok(_) => panic!("expected the request to fail"),
        Err(text) => text,
    }
}

/// `$access`, `$auth`, `$token` and `$session` are the protected parameter names in 3.2.4
/// (`surrealdb-core/src/cnf/mod.rs`, `PROTECTED_PARAM_NAMES`). Setting one with `LET`, or binding
/// it from the SDK, is refused with "is a protected variable and cannot be set". A renamed
/// variable passes (Block 04 renamed `$session` to `$count_session`). Names the engine defines
/// itself in some contexts, such as `$value` and `$this`, are not protected: `LET` accepts them,
/// so a collision there shadows silently instead of failing.
#[tokio::test]
async fn the_four_protected_parameter_names_are_refused_by_let_and_by_bind() -> Result<(), String> {
    let db = memory_db().await?;
    for name in ["access", "auth", "token", "session"] {
        let err = error_text(run(&db, &format!("LET ${name} = 1; RETURN ${name};")).await);
        assert!(
            err.contains(&format!(
                "'{name}' is a protected variable and cannot be set"
            )),
            "LET ${name}: {err}"
        );

        let bound = match db.query(format!("RETURN ${name};")).bind((name, 1)).await {
            Ok(response) => response.check().map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        };
        let err = error_text(bound);
        assert!(err.contains("protected variable"), "bind ${name}: {err}");
    }

    let mut ok = run(&db, "LET $count_session = 7; RETURN $count_session;").await?;
    let value: Option<i64> = ok.take(1).map_err(|e| e.to_string())?;
    assert_eq!(value, Some(7));

    for name in ["value", "this"] {
        let mut ok = run(&db, &format!("LET ${name} = 3; RETURN ${name};")).await?;
        let value: Option<i64> = ok.take(1).map_err(|e| e.to_string())?;
        assert_eq!(value, Some(3), "${name} is context-defined, not protected");
    }
    Ok(())
}

/// `IF … ELSE IF … ELSE … END` is one expression and closes with one `END`. A second `END` is a
/// parse error ("Unexpected token `END`, expected Eof"), and a parse error fails the whole
/// request (see `order_by_needs_its_field_selected_and_the_parse_error_fails_the_whole_request`).
#[tokio::test]
async fn an_if_else_if_chain_closes_with_one_end() -> Result<(), String> {
    let db = memory_db().await?;
    let err = error_text(
        run(
            &db,
            "RETURN IF false THEN 'a' ELSE IF true THEN 'b' ELSE 'c' END END;",
        )
        .await,
    );
    assert!(err.contains("Parse error"), "{err}");
    assert!(err.contains("Unexpected token `END`"), "{err}");

    let mut ok = run(
        &db,
        "RETURN IF false THEN 'a' ELSE IF true THEN 'b' ELSE 'c' END;",
    )
    .await?;
    let chosen: Option<String> = ok.take(0).map_err(|e| e.to_string())?;
    assert_eq!(chosen.as_deref(), Some("b"));
    Ok(())
}

#[derive(Debug, SurrealValue)]
struct RankedBucket {
    key: String,
    availability_rank: i64,
    qty: i64,
}

/// `ORDER BY` accepts idioms that appear in the projection, not expressions: an `IF` expression
/// after `ORDER BY` is a parse error. Project the rank under an alias and order by the alias;
/// the ordering semantics are unchanged.
#[tokio::test]
async fn order_by_takes_projected_idioms_not_expressions() -> Result<(), String> {
    let db = memory_db().await?;
    run(
        &db,
        "CREATE bucket:released SET status = 'RELEASED', qty = 9;
         CREATE bucket:small SET status = 'AVAILABLE', qty = 3;
         CREATE bucket:large SET status = 'AVAILABLE', qty = 5;",
    )
    .await?;

    let err = error_text(
        run(
            &db,
            "SELECT * FROM bucket ORDER BY IF status = 'AVAILABLE' THEN 0 ELSE 1 END, qty DESC;",
        )
        .await,
    );
    assert!(err.contains("Parse error"), "{err}");

    let mut ok = run(
        &db,
        "SELECT record::id(id) AS key,
                IF status = 'AVAILABLE' THEN 0 ELSE 1 END AS availability_rank,
                qty
         FROM bucket ORDER BY availability_rank, qty DESC;",
    )
    .await?;
    let rows: Vec<RankedBucket> = ok.take(0).map_err(|e| e.to_string())?;
    let keys: Vec<&str> = rows.iter().map(|row| row.key.as_str()).collect();
    assert_eq!(keys, ["large", "small", "released"]);
    assert_eq!(rows[0].availability_rank, 0);
    assert_eq!(rows[2].qty, 9);
    Ok(())
}

/// A SCHEMAFULL field `TYPE record<tx>` is required: `NONE`, or leaving the field out, is
/// refused with "Couldn't coerce value for field … Expected `record<tx>` but found `NONE`".
/// Declare `option<record<tx>>` when absence is legitimate; otherwise create the linked record
/// first and link it (Block 04 created the `handling_unit_transaction` its movement needed
/// instead of writing `transaction_id = NONE`).
#[tokio::test]
async fn a_schemafull_record_link_field_refuses_none_unless_declared_option() -> Result<(), String>
{
    let db = memory_db().await?;
    run(
        &db,
        "DEFINE TABLE tx SCHEMAFULL;
         DEFINE TABLE movement SCHEMAFULL;
         DEFINE FIELD transaction_id ON TABLE movement TYPE record<tx>;
         DEFINE TABLE movement_opt SCHEMAFULL;
         DEFINE FIELD transaction_id ON TABLE movement_opt TYPE option<record<tx>>;",
    )
    .await?;

    for statement in [
        "CREATE movement:one SET transaction_id = NONE;",
        "CREATE movement:one;",
    ] {
        let err = error_text(run(&db, statement).await);
        assert!(
            err.contains("Couldn't coerce value for field `transaction_id`"),
            "{err}"
        );
        assert!(
            err.contains("Expected `record<tx>` but found `NONE`"),
            "{err}"
        );
    }

    run(&db, "CREATE movement_opt:one SET transaction_id = NONE;").await?;
    run(
        &db,
        "CREATE tx:t1; CREATE movement:two SET transaction_id = tx:t1;",
    )
    .await?;
    Ok(())
}

/// A UNIQUE index defined over rows that lack the indexed field is accepted in 3.2.4, and rows
/// that keep lacking it are still accepted afterwards: absent values do not collide. The real
/// hazard of a migration that adds such an index is duplicates among the values the migration
/// writes, so normalize the rows first and define the index after; a duplicate then fails at the
/// index definition, with the data still consistent, instead of mid-normalization.
#[tokio::test]
async fn a_unique_index_ignores_absent_values_so_normalize_rows_before_defining_it(
) -> Result<(), String> {
    let db = memory_db().await?;
    run(
        &db,
        "CREATE task:a SET warehouse_id = 'w1';
         CREATE task:b SET warehouse_id = 'w1';
         DEFINE INDEX task_code_unique ON TABLE task FIELDS warehouse_id, task_code UNIQUE;
         CREATE task:c SET warehouse_id = 'w1';",
    )
    .await?;

    // Safe order: normalize, then index; the index enforces uniqueness from then on.
    run(
        &db,
        "CREATE task2:a SET warehouse_id = 'w1';
         CREATE task2:b SET warehouse_id = 'w1';
         UPDATE task2 SET task_code = record::id(id);
         DEFINE INDEX task2_code_unique ON TABLE task2 FIELDS warehouse_id, task_code UNIQUE;
         CREATE task2:d SET warehouse_id = 'w1', task_code = 'd';",
    )
    .await?;
    let dup = error_text(
        run(
            &db,
            "CREATE task2:c SET warehouse_id = 'w1', task_code = 'a';",
        )
        .await,
    );
    assert!(
        dup.contains("Database index `task2_code_unique` already contains"),
        "{dup}"
    );

    // Duplicates produced by the normalization fail the index definition itself.
    let collision = error_text(
        run(
            &db,
            "CREATE task3:a SET warehouse_id = 'w1', task_code = 'same';
             CREATE task3:b SET warehouse_id = 'w1', task_code = 'same';
             DEFINE INDEX task3_code_unique ON TABLE task3 FIELDS warehouse_id, task_code UNIQUE;",
        )
        .await,
    );
    println!("UNIQUE index over duplicate values -> {collision}");
    assert!(collision.contains("task3_code_unique"), "{collision}");
    Ok(())
}

/// `take_errors()` returns a `HashMap<usize, Error>`: iteration order is arbitrary. Inside a
/// failed transaction every statement carries an error. The one that failed carries the cause;
/// the statements before it say "not executed due to a failed transaction", the ones after it
/// "not executed due to a cancelled transaction", and `COMMIT` says "Cannot COMMIT: the
/// transaction was aborted due to a prior error". The cause is therefore neither the first
/// index nor whatever `.into_iter().next()` yields: report the lowest index whose message is not
/// one of the three cascade texts.
#[tokio::test]
async fn take_errors_is_unordered_and_the_cause_hides_among_cascade_errors() -> Result<(), String> {
    let db = memory_db().await?;
    run(&db, "DEFINE TABLE t SCHEMALESS;").await?;
    let mut response = db
        .query(
            "BEGIN;
             CREATE t:one SET n = 1;
             THROW 'BUSINESS_RULE_X';
             CREATE t:two SET n = 2;
             COMMIT;",
        )
        .await
        .map_err(|e| e.to_string())?;
    let errors: HashMap<usize, surrealdb::Error> = response.take_errors();
    let mut indexes: Vec<usize> = errors.keys().copied().collect();
    indexes.sort_unstable();
    assert_eq!(
        indexes,
        [1, 2, 3, 4],
        "BEGIN is statement 0 and carries no error"
    );
    assert!(
        errors[&1]
            .to_string()
            .contains("not executed due to a failed transaction"),
        "{}",
        errors[&1]
    );
    assert!(
        errors[&2].to_string().contains("BUSINESS_RULE_X"),
        "{}",
        errors[&2]
    );
    assert!(
        errors[&3]
            .to_string()
            .contains("not executed due to a cancelled transaction"),
        "{}",
        errors[&3]
    );
    assert!(
        errors[&4].to_string().contains("Cannot COMMIT"),
        "{}",
        errors[&4]
    );

    let is_cascade = |text: &str| {
        text.contains("not executed due to a failed transaction")
            || text.contains("not executed due to a cancelled transaction")
            || text.contains("Cannot COMMIT: the transaction was aborted")
    };
    let cause = indexes
        .iter()
        .find(|index| !is_cascade(&errors[index].to_string()))
        .expect("one statement carries the cause");
    assert_eq!(*cause, 2);

    let rolled_back: Vec<i64> = run(&db, "SELECT VALUE n FROM t;")
        .await?
        .take(0)
        .map_err(|e| e.to_string())?;
    assert!(
        rolled_back.is_empty(),
        "the transaction must leave nothing behind"
    );
    Ok(())
}

/// `RecordId` has public `table` and `key` fields and implements `ToSql` (the `table:key` text),
/// but not `Display`: `record_id.to_string()` does not compile in 3.2.4 (E0599 in the proving
/// repository). For the bare key use `record::id()` in SQL, or the key's `to_sql()` in Rust.
#[tokio::test]
async fn record_id_text_comes_from_record_id_in_sql_or_to_sql_in_rust() -> Result<(), String> {
    let db = memory_db().await?;
    let mut response = run(
        &db,
        "CREATE item:sku1;
         SELECT VALUE record::id(id) FROM item;
         SELECT VALUE id FROM item;",
    )
    .await?;
    let keys: Vec<String> = response.take(1).map_err(|e| e.to_string())?;
    assert_eq!(keys, ["sku1"]);
    let ids: Vec<RecordId> = response.take(2).map_err(|e| e.to_string())?;
    assert_eq!(ids.len(), 1);
    println!(
        "to_sql -> {} / key -> {}",
        ids[0].to_sql(),
        ids[0].key.to_sql()
    );
    assert_eq!(ids[0].to_sql(), "item:sku1");
    assert_eq!(ids[0].key.to_sql(), "sku1");
    Ok(())
}
