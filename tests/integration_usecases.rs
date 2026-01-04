use std::fs;
use std::io::Cursor;

use transaction_parser::domain::ledger::Ledger;

fn run_case(input_csv: &str) -> String {
    let mut ledger = Ledger::new();
    let mut worker = transaction_parser::worker::processor::Processor::new();

    let rdr = Cursor::new(input_csv.as_bytes());
    let mut csv_reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(rdr);

    for row in transaction_parser::io::reader::read_transactions(&mut csv_reader) {
        let ev = row.expect("failed to parse input row");
        worker.process(&mut ledger, ev);
    }

    let mut out = Vec::<u8>::new();
    transaction_parser::io::writer::write_accounts(&mut out, ledger.accounts())
        .expect("failed to write output CSV");
    String::from_utf8(out).expect("output was not valid UTF-8")
}

fn normalize_csv(s: &str) -> String {
    // Normalize line endings + trim trailing whitespace lines.
    // Also allows tests to be stable across platforms.
    s.replace("\r\n", "\n")
        .lines()
        .map(|l| l.trim_end())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn case1_basic_deposit_withdraw() {
    let input = fs::read_to_string("tests/fixtures/case1_input.csv").unwrap();
    let expected = fs::read_to_string("tests/fixtures/case1_expected.csv").unwrap();

    let actual = run_case(&input);

    assert_eq!(normalize_csv(&actual), normalize_csv(&expected));
}

#[test]
fn case2_dispute_resolve_chargeback_locks() {
    let input = fs::read_to_string("tests/fixtures/case2_input.csv").unwrap();
    let expected = fs::read_to_string("tests/fixtures/case2_expected.csv").unwrap();

    let actual = run_case(&input);

    assert_eq!(normalize_csv(&actual), normalize_csv(&expected));
}

#[test]
fn case3_floating_point_precision() {
    let input = fs::read_to_string("tests/fixtures/case3_input.csv").unwrap();
    let expected = fs::read_to_string("tests/fixtures/case3_expected.csv").unwrap();

    let actual = run_case(&input);

    assert_eq!(normalize_csv(&actual), normalize_csv(&expected));
}
