mod common;

use common::Test;

#[tokio::test]
async fn duplicate_data() {
    Test::for_input(
        r#"type, client, tx, amount
                deposit, 1, 1, 4.1
                deposit, 1, 1, 3.5
                "#,
    )
    .expect_error("Invalid event: Transaction already exist but with different details")
    .await;
}

/// Client 2 is trying to deposit to transaction 1 which belongs to client 1
#[tokio::test]
async fn transaction_belong_to_another_client() {
    Test::for_input(
        r#"type, client, tx, amount
                deposit, 1, 1, 4.1
                deposit, 2, 1, 3.5
                "#,
    )
    .expect_error("Invalid event: Transaction belong to a different client")
    .await;
}
