use std::collections::HashMap;

use crate::domain::{account::Account, transaction::TransactionRecord};

#[derive(Debug, Default)]
pub struct Ledger {
    pub accounts: HashMap<u16, Account>,
    pub txs: HashMap<u32, TransactionRecord>,
}
impl Ledger {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            txs: HashMap::new(),
        }
    }
    pub fn accounts(&self) -> &HashMap<u16, Account> {
        &self.accounts
    }

    pub fn get_or_create_account(&mut self, client_id: u16) -> &mut Account {
        self.accounts.entry(client_id).or_insert_with(Account::new)
    }
}
