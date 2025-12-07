use std::path::PathBuf;

use anyhow::Context;
use payment_engine::{Engine, Event, errors::EngineError, ledger::Ledger};

use crate::app::models::{InputRow, OutputRow};

pub mod models;

#[derive(Debug, Default)]
pub struct App {}

impl App {
    pub fn new() -> Self {
        App {}
    }

    pub async fn process(&self, ledger: impl Ledger, input: PathBuf) -> anyhow::Result<()> {
        // csv crate also supports reading for a reader (that impls io::Read)
        // So this could be used to stream large data from network or any other sources.
        let csv_reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All) // allow whitespaces in csv header and fields
            .flexible(true) // dispute/resolve/chargeback won't have the amount field
            .from_path(input)
            .context("Read the provided csv file")?;

        let mut engine = Engine::new(ledger);
        process_transactions_from_csv(&mut engine, csv_reader).await?;

        let accounts = engine.accounts();
        print_accounts(accounts)?;

        Ok(())
    }
}

async fn process_transactions_from_csv(
    engine: &mut Engine<impl Ledger>,
    mut reader: csv::Reader<std::fs::File>,
) -> anyhow::Result<()> {
    for entry in reader.deserialize::<InputRow>() {
        match entry {
            Ok(entry) => {
                let event = match Event::try_from(entry) {
                    Ok(event) => event,
                    Err(err) => {
                        eprintln!("Error parsing entry: {}", err);
                        continue;
                    }
                };

                match engine.apply(event).await {
                    Ok(_) => (),
                    Err(err) => match err {
                        EngineError::InvalidAssociatedTransaction(_)
                        | EngineError::InsufficientFunds
                        | EngineError::InvalidTransactionStatus(_)
                        | EngineError::DuplicateEvent
                        | EngineError::AccountLocked(_)
                        | EngineError::InvalidEvent(_) => {
                            eprintln!(
                                "Partner Data Error for TxId: {}, ClientId: {}: {}",
                                entry.tx, entry.client, err
                            )
                        }
                        EngineError::SystemError(error) => {
                            anyhow::bail!("System Error: {error}");
                        }
                    },
                }
            }
            Err(err) => eprintln!("Error reading entry: {}", err),
        }
    }

    Ok(())
}

fn print_accounts<'a>(
    accounts: impl Iterator<Item = &'a payment_engine::ClientAccount>,
) -> Result<(), anyhow::Error> {
    let mut w = csv::Writer::from_writer(std::io::stdout());
    for a in accounts {
        w.serialize(OutputRow::from(a))?;
    }
    w.flush()?;
    Ok(())
}
