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

    // disputes only apply to deposits
    if tx_type != TxType::Deposit {
        return Ok(());
    }

    // current status must be Normal for applying dispute
    if tx_status != TxStatus::Normal {
        return Ok(());
    }

    apply_dispute(ledger.get_or_create_account(client), amount);
    if let Some(t) = ledger.txs.get_mut(&tx) {
        t.set_status(TxStatus::Disputed);
    }

    Ok(())
}

fn apply_dispute(acc: &mut Account, amount: Money) {
    acc.available -= amount;
    acc.held += amount;
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::domain::transaction::TransactionRecord;

    #[test]
    fn test_handle_dispute_success() {
        let mut ledger = Ledger::default();
        let client_id = 1;
        let tx_id = 100;
        let amount = Money::from_str("50.0").unwrap();

        // Setup: Create a deposit transaction
        let tx =
            TransactionRecord::new(tx_id, client_id, amount, TxType::Deposit, TxStatus::Normal);
        ledger.txs.insert(tx_id, tx);

        // Setup: Ensure account has funds (deposit usually adds funds, simulating that state)
        let account = ledger.get_or_create_account(client_id);
        account.available = amount;

        // Act
        let result = handle(&mut ledger, client_id, tx_id);

        // Assert
        assert!(result.is_ok());

        let account = ledger.get_or_create_account(client_id);
        assert_eq!(account.available, Money::from_str("0.0").unwrap()); // Funds moved from available
        assert_eq!(account.held, amount); // To held

        let tx = ledger.txs.get(&tx_id).unwrap();
        assert_eq!(tx.tx_status, TxStatus::Disputed);
    }

    #[test]
    fn test_handle_dispute_account_locked() {
        let mut ledger = Ledger::default();
        let client_id = 1;
        let tx_id = 100;

        let account = ledger.get_or_create_account(client_id);
        account.locked = true;

        let result = handle(&mut ledger, client_id, tx_id);
        assert!(result.is_ok()); // Should return Ok(()) early
    }

    #[test]
    fn test_handle_dispute_tx_not_found() {
        let mut ledger = Ledger::default();
        let client_id = 1;
        let tx_id = 100;

        let result = handle(&mut ledger, client_id, tx_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_dispute_client_mismatch() {
        let mut ledger = Ledger::default();
        let client_id = 1;
        let other_client_id = 2;
        let tx_id = 100;
        let amount = Money::from_str("10.0").unwrap();

        let tx = TransactionRecord::new(
            tx_id,
            other_client_id,
            amount,
            TxType::Deposit,
            TxStatus::Normal,
        );
        ledger.txs.insert(tx_id, tx);

        let result = handle(&mut ledger, client_id, tx_id);
        assert!(result.is_ok());

        let tx = ledger.txs.get(&tx_id).unwrap();
        assert_eq!(tx.tx_status, TxStatus::Normal); // Status unchanged
    }

    #[test]
    fn test_handle_dispute_not_deposit() {
        let mut ledger = Ledger::default();
        let client_id = 1;
        let tx_id = 100;
        let amount = Money::from_str("10.0").unwrap();

        let tx = TransactionRecord::new(
            tx_id,
            client_id,
            amount,
            TxType::Withdrawal,
            TxStatus::Normal,
        );
        ledger.txs.insert(tx_id, tx);

        let result = handle(&mut ledger, client_id, tx_id);
        assert!(result.is_ok());

        let tx = ledger.txs.get(&tx_id).unwrap();
        assert_eq!(tx.tx_status, TxStatus::Normal);
    }

    #[test]
    fn test_handle_dispute_already_disputed() {
        let mut ledger = Ledger::default();
        let client_id = 1;
        let tx_id = 100;
        let amount = Money::from_str("10.0").unwrap();

        let mut tx =
            TransactionRecord::new(tx_id, client_id, amount, TxType::Deposit, TxStatus::Normal);
        tx.set_status(TxStatus::Disputed);
        ledger.txs.insert(tx_id, tx);

        let result = handle(&mut ledger, client_id, tx_id);
        assert!(result.is_ok());

        // Account balances should not change again if logic prevents re-disputing
        // (The current implementation checks for TxStatus::Normal, so it returns early)
        let account = ledger.get_or_create_account(client_id);
        assert_eq!(account.held, Money::from_str("0.0").unwrap());
    }

    #[test]
    fn test_handle_insufficient_funds() {
        let mut ledger = Ledger::default();
        let client_id = 1;
        let tx_id = 100;
        let amount = Money::from_str("100.0").unwrap();

        let tx =
            TransactionRecord::new(tx_id, client_id, amount, TxType::Deposit, TxStatus::Normal);
        ledger.txs.insert(tx_id, tx);

        // Account has 0 available
        let result = handle(&mut ledger, client_id, tx_id);
        assert!(result.is_ok());

        let account = ledger.get_or_create_account(client_id);

        // Available can go negative; held equals the disputed amount; tx becomes Disputed
        assert_eq!(account.available, Money::from_str("-100.0").unwrap());
        assert_eq!(account.held, amount);

        let tx = ledger.txs.get(&tx_id).unwrap();
        assert_eq!(tx.tx_status, TxStatus::Disputed);
    }
}
