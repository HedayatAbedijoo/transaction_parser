use std::{collections::HashMap, io::Write};

use crate::domain::account::Account;

#[derive(serde::Serialize)]
/// Internal CSV output row representation matching the required output headers.
///
/// Headers written (in this order): `client,available,held,total,locked`.
/// Monetary fields are formatted to 4 decimal places as strings.
struct OutputRow {
    client: u16,
    available: String,
    held: String,
    total: String,
    locked: bool,
}

/// Writes account states to a CSV writer.
///
/// The output includes a header row: `client,available,held,total,locked`.
/// For deterministic output, accounts are sorted by client id ascending before writing.
///
/// Monetary fields are formatted with exactly 4 decimal places using
/// `to_string_4dp()`.
///
/// # Errors
///
/// Returns a `csv::Error` if writing/serializing any row fails.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use transaction_parser::io::writer::write_accounts;
/// use transaction_parser::domain::account::Account;
///
/// let mut accounts = HashMap::new();
/// accounts.insert(2, Account::default());
/// accounts.insert(1, Account::default());
///
/// let mut out = Vec::new();
/// write_accounts(&mut out, &accounts).unwrap();
///
/// let s = String::from_utf8(out).unwrap();
/// assert!(s.starts_with("client,available,held,total,locked\n"));
/// // and rows are sorted by client id
/// assert!(s.contains("\n1,"));
/// assert!(s.contains("\n2,"));
/// ```
pub fn write_accounts<W: Write>(
    writer: W,
    accounts: &HashMap<u16, Account>,
) -> Result<(), csv::Error> {
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(true)
        .from_writer(writer);

    // Deterministic output: sort by client id.
    let mut clients: Vec<u16> = accounts.keys().copied().collect();
    clients.sort_unstable();

    for client in clients {
        let acc = accounts.get(&client).expect("client exists");
        let row = OutputRow {
            client,
            available: acc.available.to_string_4dp(),
            held: acc.held.to_string_4dp(),
            total: acc.total().to_string_4dp(),
            locked: acc.locked,
        };
        wtr.serialize(row)?;
    }

    wtr.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashMap, str::FromStr};

    // Helper: writes accounts to a Vec<u8> and returns UTF-8 string.
    fn write_to_string(accounts: &HashMap<u16, Account>) -> String {
        let mut out = Vec::new();
        write_accounts(&mut out, accounts).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn writes_header_and_rows_in_sorted_client_order() {
        // Create accounts inserted in non-sorted order to prove deterministic sorting.
        let mut accounts = HashMap::new();

        // NOTE: This assumes `Account::default()` exists and that its money fields
        // format to "0.0000". If your `Account` doesn't implement Default, replace
        // these with the appropriate constructor for your type.
        let mut acc_2 = Account::default();
        acc_2.locked = true;

        let mut acc_1 = Account::default();
        acc_1.locked = false;

        accounts.insert(2, acc_2);
        accounts.insert(1, acc_1);

        let s = write_to_string(&accounts);

        // Header must be present.
        assert!(s.starts_with("client,available,held,total,locked\n"));

        // Split lines to check order and exact columns.
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines.len(), 3, "expected header + 2 rows");

        // Row 1 should be client 1, row 2 should be client 2.
        // Also verify 4dp formatting + locked boolean string.
        assert_eq!(lines[1], "1,0.0000,0.0000,0.0000,false");
        assert_eq!(lines[2], "2,0.0000,0.0000,0.0000,true");
    }

    #[test]
    fn writes_total_as_available_plus_held_formatting_4dp() {
        // This test verifies `total` is derived from `acc.total()` and serialized with 4dp.
        // If Account exposes ways to set balances, use them here. The below assumes you
        // can directly assign `available` and `held` money fields.
        let mut accounts = HashMap::new();

        let mut acc = Account::default();

        // If your Account type doesn't allow direct field access, replace with
        // your domain methods (e.g., acc.available = Money::from_str("1.2500")?...).
        acc.available = crate::common::money::Money::from_str("1.2500").unwrap();
        acc.held = crate::common::money::Money::from_str("0.5000").unwrap();
        acc.locked = false;

        accounts.insert(7, acc);

        let s = write_to_string(&accounts);
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines.len(), 2, "expected header + 1 row");

        // total should be 1.7500 if total() = available + held.
        assert_eq!(lines[1], "7,1.2500,0.5000,1.7500,false");
    }
}
