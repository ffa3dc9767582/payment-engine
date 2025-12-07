use crate::engine::types::{Amount, ClientId};

#[derive(Debug)]
pub struct ClientAccount {
    pub client_id: ClientId,
    pub available: Amount,
    pub total: Amount,
    pub is_locked: bool,
}

impl ClientAccount {
    pub fn held(&self) -> Amount {
        // SAFETY: The application ensures available amount is always <= total amount
        // So this should never go negative.
        let mut total = self.total;
        total
            .try_subtract(self.available)
            .expect("Held amount calculation below zero");
        total
    }
}
