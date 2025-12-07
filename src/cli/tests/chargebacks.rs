mod common;

use common::Test;

/// dispute is resulted in chargeback, fund is returned to client and account is locked
#[tokio::test]
async fn dispute_chargeback() {
    Test::for_input(
        r#"type, client, tx, amount
                deposit, 1, 1, 3.0
                dispute, 1, 1
                chargeback, 1, 1
            "#,
    )
    .expect_output(
        r#"client,available,held,total,locked
            1,0,0,0,true
            "#,
    )
    .await;
}

/// transaction must be disputed before chargeback
#[tokio::test]
async fn chargeback_without_dispute() {
    Test::for_input(
            r#"type, client, tx, amount
                deposit, 1, 1, 3.0
                chargeback, 1, 1"#).expect_error(
            "Transaction is in invalid status: Operation doesn't apply to this transaction. Transition from Settled to ChargedBack",
        )
        .await;
}

/// once an account is locked, not further activity is allowed
#[tokio::test]
async fn locked_account_activity() {
    Test::for_input(
        r#"type, client, tx, amount
                deposit, 1, 1, 3.0
                dispute, 1, 1
                chargeback, 1, 1
                deposit, 1, 2, 20.0
            "#,
    )
    .expect_error("Client 1 account is locked, no further activity is allowed")
    .await;
}
