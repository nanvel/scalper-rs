use rust_decimal::Decimal;

#[derive(Copy, Clone)]
pub enum AlertTriggerType {
    Gte,
    Lte,
}

#[derive(Copy, Clone)]
pub struct Alert {
    pub trigger_type: AlertTriggerType,
    pub price: Decimal,
}

pub struct Alerts {
    pub alerts: Vec<Alert>,
    pub last_triggered: Option<Alert>,
}

impl Alerts {
    pub fn new() -> Self {
        Self {
            alerts: Vec::<Alert>::new(),
            last_triggered: None,
        }
    }

    pub fn add_alert(&mut self, price: Decimal, trigger_type: AlertTriggerType) {
        let alert = Alert {
            trigger_type,
            price,
        };
        self.alerts.push(alert);
    }

    pub fn scan(&mut self, bid: Decimal, ask: Decimal) -> Vec<Alert> {
        let mut triggered_alerts = Vec::new();
        self.alerts.retain(|alert| {
            let triggered = match alert.trigger_type {
                AlertTriggerType::Gte => ask >= alert.price,
                AlertTriggerType::Lte => bid <= alert.price,
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
