use rust_decimal::Decimal;

#[derive(Copy, Clone)]
pub enum TriggerType {
    Gte,
    Lte,
}

#[derive(Copy, Clone)]
pub struct PriceAlert {
    pub trigger_type: TriggerType,
    pub price: Decimal,
}

pub struct PriceAlerts {
    pub alerts: Vec<PriceAlert>,
    pub last_triggered: Option<PriceAlert>,
}

impl PriceAlerts {
    pub fn new() -> Self {
        Self {
            alerts: Vec::<PriceAlert>::new(),
            last_triggered: None,
        }
    }

    pub fn add_alert(&mut self, price: Decimal, trigger_type: TriggerType) {
        let alert = PriceAlert {
            trigger_type,
            price,
        };
        self.alerts.push(alert);
    }

    pub fn scan(&mut self, bid: Decimal, ask: Decimal) -> Vec<PriceAlert> {
        let mut triggered_alerts = Vec::new();
        self.alerts.retain(|alert| {
            let triggered = match alert.trigger_type {
                TriggerType::Gte => ask >= alert.price,
                TriggerType::Lte => bid <= alert.price,
            };
            if triggered {
                self.last_triggered = Some(alert.clone());
                triggered_alerts.push(alert.clone());
            }
            !triggered
        });

        triggered_alerts
    }

    pub fn clear(&mut self) {
        self.alerts.clear();
        self.last_triggered = None;
    }
}
