use std::io::{stdout, BufWriter};

use crate::{
    common::error::AppError,
    domain::ledger::Ledger,
    io::{reader, writer},
};

pub fn run<I, S>(args: I) -> Result<(), AppError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args: Vec<String> = args.into_iter().map(|s| s.into()).collect();
    if args.len() < 2 {
        return Err(AppError::MissingArg);
    }
    let input_path = &args[1];

    let file = std::fs::File::open(input_path)?;
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(file);
    let transactions = reader::read_transactions(&mut reader);

    let mut ledger = Ledger::new();
    let mut processor = crate::worker::processor::Processor::new();

    for event in transactions {
        let event = event.map_err(AppError::Parse)?;
        processor.process(&mut ledger, event)?;
    }

    // After processing all transactions, write the ledger state to stdout
    let stdout = stdout();
    let writer = BufWriter::new(stdout.lock());
    writer::write_accounts(writer, ledger.accounts())?;

    Ok(())
}
