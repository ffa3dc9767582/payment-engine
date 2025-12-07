use std::collections::HashMap;

use super::*;

#[derive(Debug)]
pub struct InMemoryLedger {
    transactions: HashMap<(ClientId, TransactionId), Transaction>,
    transaction_id_client_id: HashMap<TransactionId, ClientId>,
}

impl Default for InMemoryLedger {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryLedger {
    pub fn new() -> Self {
        Self {
            transactions: <_>::default(),
            transaction_id_client_id: <_>::default(),
        }
    }

    fn transaction_belong_to_different_client(
        &self,
        transaction: Transaction,
        client_id: ClientId,
    ) -> bool {
        self.transaction_id_client_id
            .get(&transaction.info().id)
            .map(|c| client_id != *c)
            .unwrap_or_default()
    }
}

impl Ledger for InMemoryLedger {
    async fn add(
        &mut self,
        client_id: ClientId,
        transaction: Transaction,
    ) -> Result<(), LedgerError> {
        let existing = self.transactions.get(&(client_id, transaction.info().id));

        match existing {
            None if self.transaction_belong_to_different_client(transaction, client_id) => Err(
                LedgerError::Conflict("Transaction belong to a different client"),
            ),
            Some(existing) => {
                if existing == &transaction {
                    Err(LedgerError::AlreadyExists)
                } else {
                    Err(LedgerError::Conflict(
                        "Transaction already exist but with different details",
                    ))
                }
            }
            None => {
                self.transaction_id_client_id
                    .insert(transaction.info().id, client_id);
                self.transactions
                    .insert((client_id, transaction.info().id), transaction);
                Ok(())
            }
        }
    }

    async fn update(
        &mut self,
        client_id: ClientId,
        transaction: Transaction,
    ) -> Result<(), LedgerError> {
        let transaction_id = transaction.info().id;
        let existing = self.transactions.get(&(client_id, transaction_id));

        match existing {
            Some(existing) if existing.info().client_id != client_id => Err(LedgerError::Conflict(
                "Transaction belong to a different client",
            )),
            Some(existing) if existing == &transaction => {
                Ok(()) // exist and identical
            }
            Some(_) | None => {
                self.transactions
                    .insert((client_id, transaction.info().id), transaction);
                Ok(())
            }
        }
    }

    async fn find(
        &self,
        client_id: ClientId,
        transaction_id: TransactionId,
    ) -> Result<Option<Transaction>, LedgerError> {
        Ok(self.transactions.get(&(client_id, transaction_id)).cloned())
    }
}

#[cfg(test)]
mod test {
    use crate::engine::types::Amount;

    use super::*;

    /// A transaction must belong to a single client.
    #[tokio::test]
    async fn transaction_id_is_globally_unique() {
        let mut ledger = InMemoryLedger::new();

        let client_a = ClientId::from(1);
        let client_b = ClientId::from(2);

        // Test insert
        let transaction = Transaction::new_settled_inbound(
            TransactionId::from(1),
            client_a,
            Amount::from_minor(100),
        );
        assert!(ledger.add(client_a, transaction).await.is_ok());

        // same transaction for different client
        let err = ledger
            .add(client_b, transaction) //<- same transaction
            .await
            .expect_err("same transaction to different client must fail");

        assert_eq!(
            err,
            LedgerError::Conflict("Transaction belong to a different client")
        );
    }
}
