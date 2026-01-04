use crate::common::money::Money;

#[derive(Debug, Clone)]
pub struct TransactionRecord {
    pub tx_id: u32,
    pub client: u16,
    pub amount: Money,
    pub tx_type: TxType,
    pub tx_status: TxStatus,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxType {
    Deposit,
    Withdrawal,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxStatus {
    Normal,
    Disputed,
    Resolved,
    ChargedBack,
}

impl TransactionRecord {
    pub fn new(
        tx_id: u32,
        client: u16,
        amount: Money,
        tx_type: TxType,
        tx_status: TxStatus,
    ) -> Self {
        Self {
            tx_id,
            client,
            amount,
            tx_type,
            tx_status,
        }
    }

    pub fn set_status(&mut self, status: TxStatus) {
        self.tx_status = status;
    }
}
