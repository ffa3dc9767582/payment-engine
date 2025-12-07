use crate::engine::types::{Amount, ClientId, TransactionId};
use crate::errors::EngineError;

pub enum Event {
    Deposit {
        client_id: ClientId,
        transaction_id: TransactionId,
        amount: Amount,
    },
    Withdraw {
        client_id: ClientId,
        transaction_id: TransactionId,
        amount: Amount,
    },
    Dispute {
        client_id: ClientId,
        transaction_id: TransactionId,
    },
    Resolve {
        client_id: ClientId,
        transaction_id: TransactionId,
    },
    Chargeback {
        client_id: ClientId,
        transaction_id: TransactionId,
    },
}

impl Event {
    /// Validate the event.
    ///
    /// Currently the only validation is to check that the amount is positive for Deposit and Withdraw events.
    pub fn validate(&self) -> Result<(), EngineError> {
        match self {
            Event::Deposit { amount, .. } | Event::Withdraw { amount, .. } => {
                if amount.as_decimal().is_sign_negative() {
                    return Err(EngineError::InvalidEvent("Amount must be positive"));
                }
            }
            _ => {}
        }

        Ok(())
    }
}
