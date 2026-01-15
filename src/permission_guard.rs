/// PermissionGuard for ERC-7715 mapping
#[derive(Debug, Clone)]
pub struct PermissionGuard {
    pub daily_limit: f64,
    pub spent_today: f64,
}

impl PermissionGuard {
    pub fn can_spend(&self, amount: f64) -> bool {
        self.spent_today + amount <= self.daily_limit
    }
    pub fn record_spend(&mut self, amount: f64) {
        self.spent_today += amount;
    }
    pub fn reset(&mut self) {
        self.spent_today = 0.0;
    }
}

// Maps to ERC-7715 JSON fields:
// - daily_limit → limit.amount
// - spent_today → tracked locally, resets per period
// - period, scope: see config/MetaMask Delegation Toolkit
