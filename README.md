# transaction_parser

A small Rust CLI application and library that parses transaction CSV files, applies business rules  
(deposits, withdrawals, disputes, resolves, chargebacks), and emits a CSV report of client balances.

---

## Quick Start

### Run (prints CSV output to `stdout`)

```sh
cargo run -- tests/fixtures/case1_input.csv
cargo run -- tests/fixtures/case2_input.csv
cargo run -- tests/fixtures/case3_input.csv
```

### Run tests

```sh
cargo test
```

---

## Project Layout

### Core

- **`src/lib.rs`** — Public module exports.
- **`src/main.rs`** — Thin binary entrypoint.
- **`src/app.rs`** — CLI wiring and application bootstrap.

### Common Utilities (`src/common/`)

- **`money.rs`** — `Money` value object and formatting helpers.
- **`event.rs`** — `TransactionEvent` enum representing parsed CSV events.
- **`error.rs`** — Centralized `AppError` type.

### Domain Model (`src/domain/`)

- **`account.rs`** — `Account` model (balances and locked state).
- **`ledger.rs`** — `Ledger` storing accounts and transaction records.
- **`transaction.rs`** — `TransactionRecord`, `TxType`, and `TxStatus`.

### IO Layer (`src/io/`)

- **`reader.rs`** — CSV parsing and input validation.
- **`writer.rs`** — CSV writer that emits results to `stdout`.

### Processing Layer (`src/worker/`)

- **`processor.rs`** — Central `process` function that routes events.
- **`handlers/`** — Per-event handlers:
  - `deposit.rs`
  - `withdrawal.rs`
  - `dispute.rs`
  - `resolve.rs`
  - `chargeback.rs`

---

## Tests and Fixtures

- **`tests/integration_usecases.rs`**  
  Integration tests that run input CSVs through the full pipeline and compare output against expected results.

- **Fixtures:** `tests/fixtures/`
  - `case1_input.csv` → `case1_expected.csv`
  - `case2_input.csv` → `case2_expected.csv`
  - `case3_input.csv` → `case3_expected.csv`

---

## AI Tools Disclosure

**GitHub Copilot** and **ChatGPT** used during the implementation.

- **GitHub Copilot:** used for boilerplate code such as well-known functions, tests, comments, fixing typos, and suggesting function/variable names during refactoring and also reviewing the code.
- **ChatGPT:** used to generate a few CSV fixture files for testing, and for technical discussion around designing an idiomatic state-machine approach in Rust.
