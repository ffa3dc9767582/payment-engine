use crate::engine::types::{Amount, ClientId, TransactionId};
use crate::errors::EngineError;
use crate::ledger::transactions::{Direction, Transaction, TransactionStatus};
use crate::{ClientAccount, Event, ledger::Ledger};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Engine<L> {
    accounts: HashMap<ClientId, ClientAccount>,
    ledger: L,
}

impl<L: Ledger> Engine<L> {
    pub fn new(ledger: L) -> Self {
        Engine {
            ledger,
            accounts: <_>::default(),
        }
    }

    /// Returns a vector of client accounts sorted by client ID.
    pub fn accounts_ordered(&self) -> Vec<&ClientAccount> {
        let mut accounts = self.accounts.values().collect::<Vec<_>>();
        accounts.sort_by_key(|c| c.client_id);
        accounts
    }

    /// Returns an iterator over client accounts.
    /// Order is not guaranteed.
    pub fn accounts(&self) -> std::collections::hash_map::Values<'_, ClientId, ClientAccount> {
        self.accounts.values()
    }

    /// Apply an event to the engine and update the associated client account.
    ///
    pub async fn apply(&mut self, event: Event) -> Result<(), EngineError> {
        event.validate()?;

        // Idempotency and atomicity considerations:
        //
        // Each of the `apply_*` methods does two things (roughly speaking)
        // 1. Add or update the transaction in the ledger.
        // 2. Update the associated client account.
        //
        // In a real system,
        // For strongly consistent systems: both operation should be done in a single transaction to ensure consistency.
        // for eventually consistent systems: we should have a mechanism to recover from partial failures.
        // It should also make sure the side effects are idempotent.
        // Although this implementation rejects same transaction being applied twice (see the state machine in `transition_inbound`)
        // There are still some edge cases: eg. withdraw failed due to insufficient funds will leave the transaction in the ledger.
        //
        // Ledger behaviour:
        // In a real system, the ledger should be immutable and append-only.
        // Therefore, we'd create compensating transactions to reverse deposit when its charged back.
        //
        // We'd also have an Event store to keep track of all the events to rebuild the client accounts later.

        match event {
            Event::Deposit {
                client_id,
                transaction_id,
                amount,
            } => {
                self.apply_deposit(client_id, transaction_id, amount)
                    .await?;
            }
            Event::Withdraw {
                client_id,
                transaction_id,
                amount,
            } => {
                self.apply_withdraw(client_id, transaction_id, amount)
                    .await?;
            }
            Event::Dispute {
                client_id,
                transaction_id,
            } => {
                self.apply_dispute(client_id, transaction_id).await?;
            }
            Event::Resolve {
                client_id,
                transaction_id,
            } => {
                self.apply_dispute_resolve(client_id, transaction_id)
                    .await?;
            }
            Event::Chargeback {
                client_id,
                transaction_id,
            } => {
                self.apply_dispute_chargeback(client_id, transaction_id)
                    .await?;
            }
        }

        Ok(())
    }

    /// Get the client account (creates a new one if it doesn't exist).
    ///
    /// If the account is locked, returns an error.
    /// This ensure that no further activity is allowed on a locked accounts.
    fn get_account_mut_ensure_unlocked(
        &mut self,
        client_id: ClientId,
    ) -> Result<&mut ClientAccount, EngineError> {
        let account = self
            .accounts
            .entry(client_id)
            .or_insert_with(|| ClientAccount {
                client_id,
                available: Amount::default(),
                total: Amount::default(),
                is_locked: false,
            });

        if account.is_locked {
            return Err(EngineError::AccountLocked(client_id));
        }

        Ok(account)
    }

    async fn apply_withdraw(
        &mut self,
        client_id: ClientId,
        transaction_id: TransactionId,
        amount: Amount,
    ) -> Result<(), EngineError> {
        let transaction = Transaction::new_settled_outbound(transaction_id, client_id, amount);

        // Limitation: Ledger entry and account update is not an atomic operation.
        // For failures due to insufficient funds, we will leave the transaction in the ledger.
        //
        // In a real app, the withdraw transaction could move to a `Failed` state.
        self.ledger.add(client_id, transaction).await?;

        let account = self.get_account_mut_ensure_unlocked(client_id)?;
        account
            .available
            .try_subtract(amount)
            .ok_or(EngineError::InsufficientFunds)?;
        account
            .total
            .try_subtract(amount)
            .ok_or(EngineError::InsufficientFunds)?; // unlikely to happen because we checked available above (which could be < total).

        Ok(())
    }

    async fn apply_deposit(
        &mut self,
        client_id: ClientId,
        transaction_id: TransactionId,
        amount: Amount,
    ) -> Result<(), EngineError> {
        let transaction = Transaction::new_settled_inbound(transaction_id, client_id, amount);

        // Limitation: lack of idempotency check.
        //
        // Workaround:
        // When the same event is replayed, This will
        // Fail with `AlreadyExists` error and prevent double counting the same transaction.
        self.ledger.add(client_id, transaction).await?;

        let account = self.get_account_mut_ensure_unlocked(client_id)?;
        account.total += amount;
        account.available += amount;

        Ok(())
    }

    async fn apply_dispute(
        &mut self,
        client_id: ClientId,
        transaction_id: TransactionId,
    ) -> Result<(), EngineError> {
        //
        // Update transaction
        //
        let mut transaction = self
            .ledger
            .find(client_id, transaction_id)
            .await?
            .ok_or(EngineError::InvalidEvent("transaction not found"))?;

        if transaction.direction() != Direction::Inbound {
            return Err(EngineError::InvalidAssociatedTransaction(
                "Dispute must be on a deposit",
            ));
        }
        // this will fail if the transaction already in `Disputed` state or in any other wrong state.
        transaction.transition_inbound(TransactionStatus::Disputed)?;

        //
        // Update Account
        //
        let account = self.get_account_mut_ensure_unlocked(client_id)?;

        account
            .available
            .try_subtract(transaction.info().amount)
            .ok_or(EngineError::InsufficientFunds)?; // for this example, we don't allow negative balance.

        self.ledger.update(client_id, transaction).await?; // update ledger when account update is successful.
        Ok(())
    }

    async fn apply_dispute_resolve(
        &mut self,
        client_id: ClientId,
        transaction_id: TransactionId,
    ) -> Result<(), EngineError> {
        //
        // Update Transaction
        //
        let mut transaction = self
            .ledger
            .find(client_id, transaction_id)
            .await?
            .ok_or(EngineError::InvalidEvent("transaction not found"))?;

        if transaction.direction() != Direction::Inbound {
            return Err(EngineError::InvalidAssociatedTransaction(
                "Dispute resolution must be on a deposit",
            ));
        }

        // this will bail if the transaction is not alredy in `Disputed` state.
        transaction.transition_inbound(TransactionStatus::Resolved)?;

        //
        // Update Account
        //
        let account = self.get_account_mut_ensure_unlocked(client_id)?;

        // release the held amount (= increase the available amount)
        account.available += transaction.info().amount;

        self.ledger.update(client_id, transaction).await?; //update ledger when account update is successful.

        Ok(())
    }

    async fn apply_dispute_chargeback(
        &mut self,
        client_id: ClientId,
        transaction_id: TransactionId,
    ) -> Result<(), EngineError> {
        let mut transaction = self
            .ledger
            .find(client_id, transaction_id)
            .await?
            .ok_or(EngineError::InvalidEvent("transaction not found"))?;

        if transaction.direction() != Direction::Inbound {
            return Err(EngineError::InvalidAssociatedTransaction(
                "Chargeback must be on a deposit",
            ));
        }

        // this will bail if the transaction is not in `Disputed` state.
        transaction.transition_inbound(TransactionStatus::ChargedBack)?;

        let account = self.get_account_mut_ensure_unlocked(client_id)?;

        // Available balance was already decreased when the transaction was disputed.
        // Now update the total amount.
        account
            .total
            .try_subtract(transaction.info().amount)
            .ok_or(EngineError::SystemError(
                "Bug: total amount should never be negative.",
            ))?;
        account.is_locked = true;

        self.ledger.update(client_id, transaction).await?; // update ledger when account update is successful.

        Ok(())
    }
}
