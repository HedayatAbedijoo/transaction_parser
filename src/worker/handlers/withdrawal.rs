use crate::{
    common::{error::AppError, money::Money},
    domain::{
        account::Account,
        ledger::Ledger,
        transaction::{TransactionRecord, TxStatus, TxType},
    },
};

pub fn handle(ledger: &mut Ledger, client: u16, tx: u32, amount: Money) -> Result<(), AppError> {
    // check if account is locked. If there are more common validations, consider moving to a common function
    if ledger.get_or_create_account(client).is_locked() {
        return Ok(());
    }

    // Check if transaction already exists (not duplicate)
    if ledger.txs.contains_key(&tx) {
        return Ok(());
    }

    apply_withdrawal(ledger.get_or_create_account(client), amount);

    ledger.txs.insert(
        tx,
        TransactionRecord {
            tx_id: tx,
            client,
            amount,
            tx_type: TxType::Withdrawal,
            tx_status: TxStatus::Normal,
        },
    );
    Ok(())
}

fn apply_withdrawal(acc: &mut Account, amount: Money) {
    if acc.available >= amount {
        acc.available -= amount;
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::domain::transaction::{TransactionRecord, TxStatus, TxType};
    // Helper to create Money from integer for tests
    fn money(v: i64) -> Money {
        Money::from_str(&v.to_string()).unwrap()
    }
    // Helper to seed an account with available funds
    fn seed_available(ledger: &mut Ledger, client: u16, available: Money) {
        let acc = ledger.get_or_create_account(client);
        acc.available = available;
    }

    #[test]
    fn handle_inserts_withdrawal_and_decreases_available_when_sufficient_funds() {
        let mut ledger = Ledger::default();

        let client = 1u16;
        let tx = 10u32;

        seed_available(&mut ledger, client, money(100));
        handle(&mut ledger, client, tx, money(40)).unwrap();

        // account changed
        let acc = ledger.get_or_create_account(client);
        assert_eq!(acc.available, money(60));

        // tx recorded correctly
        let rec = ledger.txs.get(&tx).expect("tx should be recorded");
        assert_eq!(rec.tx_id, tx);
        assert_eq!(rec.client, client);
        assert_eq!(rec.amount, money(40));
        assert_eq!(rec.tx_type, TxType::Withdrawal);
        assert_eq!(rec.tx_status, TxStatus::Normal);
    }

    #[test]
    fn handle_records_tx_even_if_insufficient_funds_but_does_not_change_available() {
        let mut ledger = Ledger::default();

        let client = 2u16;
        let tx = 11u32;

        seed_available(&mut ledger, client, money(30));
        handle(&mut ledger, client, tx, money(50)).unwrap();

        let acc = ledger.get_or_create_account(client);
        assert_eq!(acc.available, money(30), "available should not go negative");

        let rec = ledger.txs.get(&tx).expect("tx should be recorded");
        assert_eq!(rec.tx_type, TxType::Withdrawal);
        assert_eq!(rec.tx_status, TxStatus::Normal);
        assert_eq!(rec.amount, money(50));
    }

    #[test]
    fn handle_is_idempotent_for_duplicate_tx_and_does_not_apply_twice() {
        let mut ledger = Ledger::default();

        let client = 3u16;
        let tx = 12u32;

        seed_available(&mut ledger, client, money(100));
        handle(&mut ledger, client, tx, money(10)).unwrap();
        handle(&mut ledger, client, tx, money(10)).unwrap(); // duplicate

        let acc = ledger.get_or_create_account(client);
        assert_eq!(
            acc.available,
            money(90),
            "duplicate tx must not withdraw twice"
        );

        // still exactly one record for that tx id
        assert!(ledger.txs.contains_key(&tx));
    }

    #[test]
    fn handle_does_nothing_when_account_is_locked() {
        let mut ledger = Ledger::default();

        let client = 4u16;
        let tx = 13u32;

        // Seed funds and lock the account.
        {
            let acc = ledger.get_or_create_account(client);
            acc.available = money(100);

            acc.locked = true;
        }

        handle(&mut ledger, client, tx, money(20)).unwrap();

        // no balance change
        let acc = ledger.get_or_create_account(client);
        assert_eq!(acc.available, money(100));

        // no tx recorded
        assert!(
            !ledger.txs.contains_key(&tx),
            "locked account should not record withdrawals"
        );
    }

    #[test]
    fn handle_returns_ok_and_does_nothing_if_tx_already_exists_even_if_account_locked() {
        let mut ledger = Ledger::default();

        let client = 5u16;
        let tx = 14u32;

        // Insert an existing tx record first
        ledger.txs.insert(
            tx,
            TransactionRecord {
                tx_id: tx,
                client,
                amount: money(1),
                tx_type: TxType::Withdrawal,
                tx_status: TxStatus::Normal,
            },
        );

        // Lock the account and give it funds
        {
            let acc = ledger.get_or_create_account(client);
            acc.available = money(100);
            // Adjust as needed
            acc.locked = true;
        }

        // Should early-return Ok(()) due to duplicate tx
        handle(&mut ledger, client, tx, money(50)).unwrap();

        // balance unchanged
        let acc = ledger.get_or_create_account(client);
        assert_eq!(acc.available, money(100));

        // tx unchanged
        let rec = ledger.txs.get(&tx).unwrap();
        assert_eq!(rec.amount, money(1));
    }
}
