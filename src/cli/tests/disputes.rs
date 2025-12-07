mod common;

use common::Test;

/// client disputing their own transaction - should succeed
#[tokio::test]
async fn dispute() {
    Test::for_input(
        r#"type, client, tx, amount
                deposit, 1, 1, 3.0
                dispute, 1, 1
            "#,
    )
    .expect_output(
        r#"client,available,held,total,locked
            1,0,3,3,false
            "#,
    )
    .await;
}

/// Similarly, disputing a transaction when account has insufficient funds is not allowed.
/// In a real system the client balance could go negative, but for this exercise we
/// will not allow that.
#[tokio::test]
async fn dispute_insufficient_funds() {
    Test::for_input(
        r#"type, client, tx, amount
                deposit, 1, 1, 3.0
                withdrawal, 1, 2, 2.0
                dispute, 1, 1"#,
    )
    .expect_error("Insufficient funds")
    .await;
}

/// dispute a transaction that belongs to different client
#[tokio::test]
async fn dispute_wrong_transaction() {
    Test::for_input(
        r#"type, client, tx, amount
                deposit, 1, 1, 3.0
                dispute, 2, 1"#,
    )
    .expect_error("Invalid event: transaction not found") // the system hides the existence of the transaction (belong to a different client)
    .await;
}

/// dispute does not apply to withdrawals
#[tokio::test]
async fn dispute_withdrawals() {
    Test::for_input(
        r#"type, client, tx, amount
                deposit, 1, 1, 3.0
                withdrawal, 1, 2, 3.0
                dispute, 1, 2"#,
    )
    .expect_error("Invalid associated transaction: Dispute must be on a deposit")
    .await;
}
