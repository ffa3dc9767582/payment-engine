use anyhow::Context;
use payment_engine::{ClientAccount, Event};
use rust_decimal::Decimal;

#[derive(Debug, serde::Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub struct InputRow {
    #[serde(rename = "type")]
    pub ty: EntryType,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<Decimal>,
}

#[derive(Debug, serde::Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, serde::Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub struct OutputRow {
    pub client: u16,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

impl TryFrom<InputRow> for Event {
    type Error = anyhow::Error;

    fn try_from(entry: InputRow) -> anyhow::Result<Self> {
        Ok(match entry.ty {
            EntryType::Deposit => Event::Deposit {
                client_id: entry.client.into(),
                transaction_id: entry.tx.into(),
                amount: entry
                    .amount
                    .context("Amount is required for Deposit")?
                    .into(),
            },
            EntryType::Withdrawal => Event::Withdraw {
                client_id: entry.client.into(),
                transaction_id: entry.tx.into(),
                amount: entry
                    .amount
                    .context("Amount is required for Withdrawal")?
                    .into(),
            },
            EntryType::Dispute => Event::Dispute {
                client_id: entry.client.into(),
                transaction_id: entry.tx.into(),
            },
            EntryType::Resolve => Event::Resolve {
                client_id: entry.client.into(),
                transaction_id: entry.tx.into(),
            },
            EntryType::Chargeback => Event::Chargeback {
                client_id: entry.client.into(),
                transaction_id: entry.tx.into(),
            },
        })
    }
}

impl From<&ClientAccount> for OutputRow {
    fn from(account: &ClientAccount) -> Self {
        Self {
            client: account.client_id.as_inner(),
            available: account.available.as_decimal(),
            held: account.held().as_decimal(),
            total: account.total.as_decimal(),
            locked: account.is_locked,
        }
    }
}
