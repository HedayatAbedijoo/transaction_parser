use crate::{
    common::{error::AppError, money::Money},
    domain::{
        account::Account,
        ledger::Ledger,
        transaction::{TxStatus, TxType},
    },
};

pub fn handle(ledger: &mut Ledger, client: u16, tx: u32) -> Result<(), AppError> {
    // check if account is locked. If there are more common validations, consider moving to a common function
    if ledger.get_or_create_account(client).is_locked() {
        return Ok(());
    }

    let (tx_client, tx_type, tx_status, amount) = {
        match ledger.txs.get(&tx) {
            Some(t) => (t.client, t.tx_type, t.tx_status, t.amount),
            None => return Ok(()),
        }
    };

    // must match client
    if tx_client != client {
        // log the reason before exit, or send back a clear error.
        return Ok(());
    }

    // must be disputed to chargeback
    if tx_status != TxStatus::Disputed {
        // log the reason before exit, or send back a clear error.
        return Ok(());
    }

    // chargeback is typically only valid for deposit disputes
    if tx_type != TxType::Deposit {
        // log the reason before exit, or send back a clear error.
        return Ok(());
    }

    if apply_chargeback(ledger.get_or_create_account(client), amount) {
        if let Some(t) = ledger.txs.get_mut(&tx) {
            t.set_status(TxStatus::ChargedBack);
        }
    }

    Ok(())
}

fn apply_chargeback(acc: &mut Account, amount: Money) -> bool {
    if acc.held >= amount {
        acc.held -= amount;
        acc.locked = true;
        true
    } else {
        false
    }
}
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::domain::transaction::TransactionRecord;

    use super::*;

    #[test]
    fn test_handle_chargeback_success() {
        let mut ledger = Ledger::default();
        let client_id = 1;
        let tx_id = 300;
        let amount = Money::from_str("50.0").unwrap();

        // Setup: Disputed deposit
        let mut tx =
            TransactionRecord::new(tx_id, client_id, amount, TxType::Deposit, TxStatus::Normal);
        tx.set_status(TxStatus::Disputed);
        ledger.txs.insert(tx_id, tx);

        // Setup: account has the disputed amount held
        let account = ledger.get_or_create_account(client_id);
        account.available = Money::from_str("0.0").unwrap();
        account.held = amount;
        account.locked = false;

        // Act
        let result = handle(&mut ledger, client_id, tx_id);

        // Assert
        assert!(result.is_ok());

        let account = ledger.get_or_create_account(client_id);
        assert_eq!(account.held, Money::from_str("0.0").unwrap());
        assert_eq!(account.available, Money::from_str("0.0").unwrap());
        assert!(account.locked);

        let tx = ledger.txs.get(&tx_id).unwrap();
        assert_eq!(tx.tx_status, TxStatus::ChargedBack);
    }

    #[test]
    fn test_handle_chargeback_tx_not_found() {
        let mut ledger = Ledger::default();
        let result = handle(&mut ledger, 1, 300);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_chargeback_client_mismatch() {
        let mut ledger = Ledger::default();
        let client_id = 1;
        let other_client_id = 2;
        let tx_id = 300;
        let amount = Money::from_str("10.0").unwrap();

        let mut tx = TransactionRecord::new(
            tx_id,
            other_client_id,
            amount,
            TxType::Deposit,
            TxStatus::Normal,
        );
        tx.set_status(TxStatus::Disputed);
        ledger.txs.insert(tx_id, tx);

        let result = handle(&mut ledger, client_id, tx_id);
        assert!(result.is_ok());

        let tx = ledger.txs.get(&tx_id).unwrap();
        assert_eq!(tx.tx_status, TxStatus::Disputed);

        // account should not be locked (and should still be default state)
        let account = ledger.get_or_create_account(client_id);
        assert!(!account.is_locked());
    }

    #[test]
    fn test_handle_chargeback_not_disputed() {
        let mut ledger = Ledger::default();
        let client_id = 1;
        let tx_id = 300;
        let amount = Money::from_str("10.0").unwrap();

        let tx =
            TransactionRecord::new(tx_id, client_id, amount, TxType::Deposit, TxStatus::Normal);
        ledger.txs.insert(tx_id, tx);

        let result = handle(&mut ledger, client_id, tx_id);
        assert!(result.is_ok());

        let tx = ledger.txs.get(&tx_id).unwrap();
        assert_eq!(tx.tx_status, TxStatus::Normal);

        let account = ledger.get_or_create_account(client_id);
        assert!(!account.is_locked());
    }

    #[test]
    fn test_handle_chargeback_account_locked() {
        let mut ledger = Ledger::default();
        let client_id = 1;
        let tx_id = 300;
        let amount = Money::from_str("10.0").unwrap();

        let mut tx =
            TransactionRecord::new(tx_id, client_id, amount, TxType::Deposit, TxStatus::Normal);
        tx.set_status(TxStatus::Disputed);
        ledger.txs.insert(tx_id, tx);

        let account = ledger.get_or_create_account(client_id);
        account.locked = true;

        let result = handle(&mut ledger, client_id, tx_id);
        assert!(result.is_ok());

        let tx = ledger.txs.get(&tx_id).unwrap();
        assert_eq!(tx.tx_status, TxStatus::Disputed);

        // still locked
        let account = ledger.get_or_create_account(client_id);
        assert!(account.is_locked());
    }

    #[test]
    fn test_handle_chargeback_insufficient_held_funds() {
        let mut ledger = Ledger::default();
        let client_id = 1;
        let tx_id = 300;
        let amount = Money::from_str("100.0").unwrap();

        let mut tx =
            TransactionRecord::new(tx_id, client_id, amount, TxType::Deposit, TxStatus::Normal);
        tx.set_status(TxStatus::Disputed);
        ledger.txs.insert(tx_id, tx);

        // account has less held than amount
        let account = ledger.get_or_create_account(client_id);
        account.available = Money::from_str("0.0").unwrap();
        account.held = Money::from_str("20.0").unwrap();
        account.locked = false;

        let result = handle(&mut ledger, client_id, tx_id);
        assert!(result.is_ok());

        let account = ledger.get_or_create_account(client_id);
        assert_eq!(account.held, Money::from_str("20.0").unwrap());
        assert_eq!(account.available, Money::from_str("0.0").unwrap());
        assert!(!account.locked);

        // tx status should remain Disputed if apply failed
        let tx = ledger.txs.get(&tx_id).unwrap();
        assert_eq!(tx.tx_status, TxStatus::Disputed);
    }
}
