mod common;

use common::Test;

#[tokio::test]
async fn basic() {
    Test::for_input(
        r#"type, client, tx, amount
                deposit, 1, 1, 4.0
                withdrawal, 1, 2, 3.0
                "#,
    )
    .expect_output(
        r#"client,available,held,total,locked
            1,1,0,1,false
            "#,
    )
    .await;
}

/// withdrawal with insufficient funds should error
#[tokio::test]
async fn insufficient_funds() {
    Test::for_input(
        r#"type, client, tx, amount
                withdrawal, 2, 5, 3.0
                "#,
    )
    .expect_error("Insufficient funds")
    .await;
}
