mod common;

use common::Test;

/// dispute is resolved
#[tokio::test]
async fn dispute_resolve() {
    Test::for_input(
        r#"type, client, tx, amount
                deposit, 1, 1, 3.0
                dispute, 1, 1
                resolve, 1, 1
            "#,
    )
    .expect_output(
        r#"client,available,held,total,locked
            1,3,0,3,false
            "#,
    )
    .await;
}

/// transaction bust be disputed before resolving
#[tokio::test]
async fn resolve_without_dispute() {
    Test::for_input(
            r#"type, client, tx, amount
                deposit, 1, 1, 3.0
                resolve, 1, 1"#).expect_error(
            "Transaction is in invalid status: Operation doesn't apply to this transaction. Transition from Settled to Resolved",
        )
        .await;
}
