use crate::common::{event::TransactionEvent, money::Money};
use std::{io::Read, str::FromStr};

#[derive(serde::Deserialize)]
/// Internal CSV row representation matching the input headers. The amount
/// field stays empty for dispute/resolve/chargeback rows.
struct CsvRow {
    #[serde(rename = "type")]
    tx_type: String,
    client: u16,
    tx: u32,
    // amount is blank for dispute/resolve/chargeback
    amount: Option<String>,
}

/// Reads and validates transaction rows from a CSV reader.
///
/// Supported headers: `type,client,tx,amount`.
/// Normalizes the `type` field to lowercase and requires `amount` for
/// `deposit` and `withdrawal` rows; errors include client/tx context.
///
/// # Examples
///
/// ```
/// use transaction_parser::io::reader::read_transactions;
/// use transaction_parser::common::event::TransactionEvent;
/// use csv::ReaderBuilder;
///
/// let data = "type,client,tx,amount\n\
/// deposit,1,10,1.25\n\
/// withdrawal,1,11,0.25\n";
/// let mut rdr = ReaderBuilder::new().from_reader(data.as_bytes());
/// let events: Vec<_> = read_transactions(&mut rdr).collect();
///
/// assert!(matches!(events[0], Ok(TransactionEvent::Deposit { client: 1, tx: 10, .. })));
/// assert!(matches!(events[1], Ok(TransactionEvent::Withdrawal { client: 1, tx: 11, .. })));
/// ```
pub fn read_transactions<R: Read>(
    rdr: &mut csv::Reader<R>,
) -> impl Iterator<Item = Result<TransactionEvent, String>> + '_ {
    // Map each CSV row into a domain `TransactionEvent`, normalizing type
    // names and validating required amounts for deposit/withdrawal.
    rdr.deserialize::<CsvRow>().map(|res| {
        let row = res.map_err(|e| e.to_string())?;
        let kind = row.tx_type.trim().to_ascii_lowercase();

        match kind.as_str() {
            "deposit" => {
                let amt_str = row.amount.ok_or_else(|| {
                    format!(
                        "deposit missing amount for client {} tx {}",
                        row.client, row.tx
                    )
                })?;
                let amount = Money::from_str(&amt_str).map_err(|e| e.to_string())?;

                Ok(TransactionEvent::Deposit {
                    client: row.client,
                    tx: row.tx,
                    amount,
                })
            }
            "withdrawal" => {
                let amt_str = row.amount.ok_or_else(|| {
                    format!(
                        "withdrawal missing amount for client {} tx {}",
                        row.client, row.tx
                    )
                })?;
                let amount = Money::from_str(&amt_str).map_err(|e| e.to_string())?;
                Ok(TransactionEvent::Withdrawal {
                    client: row.client,
                    tx: row.tx,
                    amount,
                })
            }
            "dispute" => Ok(TransactionEvent::Dispute {
                client: row.client,
                tx: row.tx,
            }),
            "resolve" => Ok(TransactionEvent::Resolve {
                client: row.client,
                tx: row.tx,
            }),
            "chargeback" => Ok(TransactionEvent::Chargeback {
                client: row.client,
                tx: row.tx,
            }),
            other => Err(format!(
                "unknown transaction type: {other} for client {} tx {}",
                row.client, row.tx
            )),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    // Helper: parse CSV input into collected transaction events for assertions.
    fn collect_events(input: &str) -> Vec<Result<TransactionEvent, String>> {
        let mut reader = csv::ReaderBuilder::new().from_reader(input.as_bytes());
        read_transactions(&mut reader).collect()
    }

    #[test]
    fn parses_all_supported_event_types() {
        let data = "type,client,tx,amount\n\
deposit,1,1,1.5000\nwithdrawal,1,2,0.5000\ndispute,1,1,\nresolve,1,1,\nchargeback,1,1,\n";
        let events = collect_events(data);

        assert_eq!(events.len(), 5);

        let expected_deposit = Money::from_str("1.5000").unwrap().as_i64();
        let expected_withdrawal = Money::from_str("0.5000").unwrap().as_i64();

        match &events[0] {
            Ok(TransactionEvent::Deposit { client, tx, amount }) => {
                assert_eq!((*client, *tx, amount.as_i64()), (1, 1, expected_deposit));
            }
            other => panic!("unexpected deposit event: {other:?}"),
        }

        match &events[1] {
            Ok(TransactionEvent::Withdrawal { client, tx, amount }) => {
                assert_eq!((*client, *tx, amount.as_i64()), (1, 2, expected_withdrawal));
            }
            other => panic!("unexpected withdrawal event: {other:?}"),
        }

        assert!(matches!(
            events[2],
            Ok(TransactionEvent::Dispute { client: 1, tx: 1 })
        ));
        assert!(matches!(
            events[3],
            Ok(TransactionEvent::Resolve { client: 1, tx: 1 })
        ));
        assert!(matches!(
            events[4],
            Ok(TransactionEvent::Chargeback { client: 1, tx: 1 })
        ));
    }

    #[test]
    fn reports_missing_amount_error() {
        let data = "type,client,tx,amount\n\
deposit,1,1,\n";
        let events = collect_events(data);

        assert_eq!(events.len(), 1);
        let err = events.into_iter().next().unwrap().unwrap_err();
        assert_eq!(err, "deposit missing amount for client 1 tx 1");
    }

    #[test]
    fn reports_unknown_type_error() {
        let data = "type,client,tx,amount\n\nrefund,1,99,10\n";
        let events = collect_events(data);

        assert_eq!(events.len(), 1);
        let err = events.into_iter().next().unwrap().unwrap_err();
        assert_eq!(err, "unknown transaction type: refund for client 1 tx 99");
    }
}
