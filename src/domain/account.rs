use crate::common::money::Money;
#[derive(Debug, Clone, Default)]
pub struct Account {
    /// Funds available for normal use.
    pub available: Money,
    /// Funds held due to disputes.
    pub held: Money,
    /// Frozen after a chargeback.
    pub locked: bool,
}
impl Account {
    pub fn new() -> Self {
        Self {
            available: Money::zero(),
            held: Money::zero(),
            locked: false,
        }
    }

    pub fn total(&self) -> Money {
        self.available + self.held
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }
}
