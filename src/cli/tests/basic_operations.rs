mod common;

use common::Test;

#[tokio::test]
async fn empty() {
    Test::for_input(r#""#).expect_output(r#""#).await;
}

#[tokio::test]
async fn basic() {
    Test::for_input(
        r#"type, client, tx, amount
                deposit, 1, 1, 1.0
                deposit, 2, 2, 5.0
                deposit, 1, 3, 2.0
                withdrawal, 1, 4, 1.5
                withdrawal, 2, 5, 3.0
            "#,
    )
    .expect_output(
        r#"client,available,held,total,locked
            1,1.5,0.0,1.5,false
            2,2,0,2,false
            "#, // Tx-5 is skipped due to insufficient funds (ignore partner error)
    )
    .await;
}

#[tokio::test]
async fn deposit() {
    Test::for_input(
        r#"type, client, tx, amount
                deposit, 1, 1, 4.1
                deposit, 1, 2, 3.5
                "#,
    )
    .expect_output(
        r#"client,available,held,total,locked
            1,7.6,0.0,7.6,false
            "#,
    )
    .await;
}

/// Allow upto 4 decimal points
#[tokio::test]
async fn decimal_points() {
    // 4 decimal points
    Test::for_input(
        r#"type, client, tx, amount
                deposit, 1, 1, 1.1234
            "#,
    )
    .expect_output(
        r#"client,available,held,total,locked
            1,1.1234,0.0000,1.1234,false
            "#,
    )
    .await;

    // 5 decimal points, rounded to 4 decimal points
    Test::for_input(
        r#"type, client, tx, amount
                deposit, 1, 1, 1.12349
            "#,
    )
    .expect_output(
        r#"client,available,held,total,locked
            1,1.1235,0.0000,1.1235,false
            "#,
    )
    .await;
}

/// negative amount isn't allowed in input data
#[tokio::test]
async fn negative_amount() {
    Test::for_input(
        r#"type, client, tx, amount
                deposit, 1, 1, -3.0
                "#,
    )
    .expect_error("Invalid event: Amount must be positive")
    .await;
}
