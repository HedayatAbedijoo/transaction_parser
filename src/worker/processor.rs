use crate::{
    common::{error::AppError, event::TransactionEvent},
    domain::ledger::Ledger,
    worker::handlers::{chargeback, deposit, dispute, resolve, withdrawal},
};

#[derive(Debug, Default)]
pub struct Processor {}
impl Processor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn process(
        &mut self,
        ledger: &mut Ledger,
        event: TransactionEvent,
    ) -> Result<(), AppError> {
        match event {
            TransactionEvent::Deposit {
                tx: tx_id,
                client,
                amount,
            } => {
                deposit::handle(ledger, client, tx_id, amount)?;
            }
            TransactionEvent::Withdrawal {
                tx: tx_id,
                client,
                amount,
            } => {
                withdrawal::handle(ledger, client, tx_id, amount)?;
            }
            TransactionEvent::Dispute { tx: tx_id, client } => {
                dispute::handle(ledger, client, tx_id)?;
            }
            TransactionEvent::Resolve { tx: tx_id, client } => {
                resolve::handle(ledger, client, tx_id)?;
            }
            TransactionEvent::Chargeback { tx: tx_id, client } => {
                chargeback::handle(ledger, client, tx_id)?;
            }
        }
        Ok(())
    }
}
