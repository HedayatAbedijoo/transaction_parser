use crate::common::money::Money;

/// Represents a transaction event that is sent from the reader to the worker for processing.
#[derive(Debug)]
pub enum TransactionEvent {
    Deposit { client: u16, tx: u32, amount: Money },
    Withdrawal { client: u16, tx: u32, amount: Money },
    Dispute { client: u16, tx: u32 },
    Resolve { client: u16, tx: u32 },
    Chargeback { client: u16, tx: u32 },
}
