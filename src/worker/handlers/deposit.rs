use crate::{
    common::{error::AppError, money::Money},
    domain::{account::Account, ledger::Ledger},
};

pub fn handle(ledger: &mut Ledger, client: u16, tx: u32, amount: Money) -> Result<(), AppError> {
    // check if account is locked. If there are more common validations, consider moving to a common function
    if ledger.get_or_create_account(client).is_locked() {
        return Ok(());
    }

    //check if transaction already exists
    if ledger.txs.contains_key(&tx) {
        return Ok(());
    }

    apply_deposit(ledger.get_or_create_account(client), amount);

    ledger.txs.insert(
        tx,
        crate::domain::transaction::TransactionRecord {
            tx_id: tx,
            client,
            amount,
            tx_type: crate::domain::transaction::TxType::Deposit,
            tx_status: crate::domain::transaction::TxStatus::Normal,
        },
    );
    Ok(())
}

fn apply_deposit(acc: &mut Account, amount: Money) {
    acc.available += amount;
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::handle;
    use crate::{common::money::Money, domain::ledger::Ledger};

    #[test]
    fn deposit_applies_credit_and_records_tx() {
        let mut ledger = Ledger::new();

        let _ = handle(&mut ledger, 1, 10, Money::from_str("1.2500").unwrap());

        let acc = ledger.accounts().get(&1).expect("account exists");
        assert_eq!(
            acc.available.as_i64(),
            Money::from_str("1.2500").unwrap().as_i64()
        );
        assert_eq!(
            acc.held.as_i64(),
            Money::from_str("0.0000").unwrap().as_i64()
        );
        assert!(!acc.locked);

        let rec = ledger.txs.get(&10).expect("tx recorded");
        assert_eq!(rec.client, 1);
        assert_eq!(rec.tx_id, 10);
        assert_eq!(
            rec.amount.as_i64(),
            Money::from_str("1.2500").unwrap().as_i64()
        );
        assert_eq!(rec.tx_type, crate::domain::transaction::TxType::Deposit);
        assert_eq!(rec.tx_status, crate::domain::transaction::TxStatus::Normal);
    }

    #[test]
    fn deposit_ignores_duplicate_tx_id() {
        let mut ledger = Ledger::new();

        let _ = handle(&mut ledger, 1, 10, Money::from_str("1.0000").unwrap());
        let _ = handle(&mut ledger, 1, 10, Money::from_str("9.0000").unwrap()); // duplicate tx id must be ignored

        let acc = ledger.accounts().get(&1).expect("account exists");
        assert_eq!(
            acc.available.as_i64(),
            Money::from_str("1.0000").unwrap().as_i64()
        ); // unchanged

        let rec = ledger.txs.get(&10).expect("tx recorded");
        assert_eq!(
            rec.amount.as_i64(),
            Money::from_str("1.0000").unwrap().as_i64()
        ); // original remains
    }

    #[test]
    fn deposit_is_ignored_if_account_is_locked() {
        let mut ledger = Ledger::new();

        // Create account and lock it
        {
            let acc = ledger.get_or_create_account(1);
            acc.locked = true;
        }

        let _ = handle(&mut ledger, 1, 10, Money::from_str("3.0000").unwrap());

        let acc = ledger.accounts().get(&1).expect("account exists");
        assert_eq!(
            acc.available.as_i64(),
            Money::from_str("0.0000").unwrap().as_i64()
        );
        assert_eq!(
            acc.held.as_i64(),
            Money::from_str("0.0000").unwrap().as_i64()
        );
        assert!(acc.locked);

        // Important: should NOT record tx when ignored due to lock
        assert!(ledger.txs.get(&10).is_none());
    }
}
