#![allow(dead_code)]

use payment_engine::{
    Engine, Event,
    ledger::in_memory::{self, InMemoryLedger},
};
use payment_engine_cli::app::models::{InputRow, OutputRow};
use pretty_assertions::assert_eq;

pub struct Test {
    input: &'static str,
}

impl Test {
    pub fn for_input(input: &'static str) -> Self {
        Self { input }
    }

    pub async fn expect_output(self, output: &'static str) {
        let engine = Self::process_csv(self.input).await.expect("process csv");

        let accounts = engine.accounts_ordered();
        let mut w = csv::Writer::from_writer(Vec::new());
        for a in accounts {
            w.serialize(OutputRow::from(a)).expect("process output row");
        }
        w.flush().expect("flush to output");

        let output = output
            .split('\n')
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join("\n");
        let csv = String::from_utf8(w.into_inner().expect("read the buffer"))
            .expect("convert vec to utf8 string");
        assert_eq!(csv, output);
    }

    pub async fn expect_error(self, error: &'static str) {
        match Self::process_csv(self.input).await {
            Err(err) => assert_eq!(err.to_string(), error),
            Ok(_) => panic!("Expected an error but got success"),
        }
    }

    async fn process_csv(input: &str) -> anyhow::Result<Engine<InMemoryLedger>> {
        let mut reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .flexible(true)
            .from_reader(input.trim().trim_end_matches(['\n', '\r']).as_bytes());

        let mut engine = Engine::new(in_memory::InMemoryLedger::new());
        for entry in reader.deserialize::<InputRow>() {
            let entry = entry?;
            engine
                .apply(Event::try_from(entry).expect("valid csv data feed"))
                .await
                .inspect_err(|e| {
                    eprintln!("Error processing {:?}: {e}", entry);
                })?;
        }

        Ok(engine)
    }
}
