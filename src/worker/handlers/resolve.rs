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
        return Ok(());
    }

    // must be disputed to resolve
    if tx_status != TxStatus::Disputed {
        return Ok(());
    }

    // resolve should only apply to deposit disputes
    if tx_type != TxType::Deposit {
        return Ok(());
    }

    if apply_resolve(ledger.get_or_create_account(client), amount) {
        if let Some(t) = ledger.txs.get_mut(&tx) {
            t.set_status(TxStatus::Resolved);
        }
    }

    Ok(())
}

fn apply_resolve(acc: &mut Account, amount: Money) -> bool {
    // Resolve: held -> available
    if acc.held >= amount {
        acc.held -= amount;
        acc.available += amount;
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::domain::transaction::TransactionRecord;

    #[test]
    fn test_handle_resolve_success() {
        let mut ledger = Ledger::default();
        let client_id = 1;
        let tx_id = 200;
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

        // Act
        let result = handle(&mut ledger, client_id, tx_id);

        // Assert
        assert!(result.is_ok());

        let account = ledger.get_or_create_account(client_id);
        assert_eq!(account.held, Money::from_str("0.0").unwrap());
        assert_eq!(account.available, amount);

        let tx = ledger.txs.get(&tx_id).unwrap();
        assert_eq!(tx.tx_status, TxStatus::Resolved);
    }

    #[test]
    fn test_handle_resolve_account_locked() {
        let mut ledger = Ledger::default();
        let client_id = 1;
        let tx_id = 200;

        let account = ledger.get_or_create_account(client_id);
        account.locked = true;

        let result = handle(&mut ledger, client_id, tx_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_resolve_tx_not_found() {
        let mut ledger = Ledger::default();
        let client_id = 1;
        let tx_id = 200;

        let result = handle(&mut ledger, client_id, tx_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_resolve_client_mismatch() {
        let mut ledger = Ledger::default();
        let client_id = 1;
        let other_client_id = 2;
        let tx_id = 200;
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
    }

    #[test]
    fn test_handle_resolve_not_disputed() {
        let mut ledger = Ledger::default();
        let client_id = 1;
        let tx_id = 200;
        let amount = Money::from_str("10.0").unwrap();

        let tx =
            TransactionRecord::new(tx_id, client_id, amount, TxType::Deposit, TxStatus::Normal);
        ledger.txs.insert(tx_id, tx);

        let result = handle(&mut ledger, client_id, tx_id);
        assert!(result.is_ok());

        let tx = ledger.txs.get(&tx_id).unwrap();
        assert_eq!(tx.tx_status, TxStatus::Normal);
    }

    #[test]
    fn test_handle_resolve_insufficient_held_funds() {
        let mut ledger = Ledger::default();
        let client_id = 1;
        let tx_id = 200;
        let amount = Money::from_str("100.0").unwrap();

        let mut tx =
            TransactionRecord::new(tx_id, client_id, amount, TxType::Deposit, TxStatus::Normal);
        tx.set_status(TxStatus::Disputed);
        ledger.txs.insert(tx_id, tx);

        // account has less held than amount
        let account = ledger.get_or_create_account(client_id);
        account.available = Money::from_str("0.0").unwrap();
        account.held = Money::from_str("20.0").unwrap();

        let result = handle(&mut ledger, client_id, tx_id);
        assert!(result.is_ok());

        let account = ledger.get_or_create_account(client_id);
        assert_eq!(account.held, Money::from_str("20.0").unwrap());
        assert_eq!(account.available, Money::from_str("0.0").unwrap());

        // tx status should remain Disputed if apply failed
        let tx = ledger.txs.get(&tx_id).unwrap();
        assert_eq!(tx.tx_status, TxStatus::Disputed);
    }
}
