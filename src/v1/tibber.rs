use std::fmt::Display;

#[derive(Debug)]
pub struct SubscriptionQuery {
    pub token: String,
    pub home_id: String,
    pub fields: String,
}

impl SubscriptionQuery {
    pub fn new(token: String, home_id: String, fields: String) -> Self {
        Self {
            token,
            home_id,
            fields,
        }
    }

    pub fn connection(&self) -> String {
        format!(
            "{{\"type\":\"connection_init\",\"payload\":{{\"token\":\"{}\"}}}}",
            self.token
        )
    }

    pub fn subscription(&self) -> String {
        format!(
            "{{\"id\":\"1\",\"type\":\"subscribe\",\"payload\":{{\"variables\":{{}},\"extensions\":{{}},\"query\":\"subscription {{\\n  liveMeasurement(homeId: \\\"{}\\\") {{\\n    {}\\n}}\\n}}\\n\"}}}}",
            self.home_id,
            self.fields
        )
    }
}

pub struct SubscriptionQueryBuilder {
    token: String,
    home_id: String,
    fields: Vec<Field>,
}

impl SubscriptionQueryBuilder {
    pub fn new(token: String, home_id: String) -> SubscriptionQueryBuilder {
        SubscriptionQueryBuilder {
            token,
            home_id,
            fields: Vec::new(),
        }
    }

    pub fn with(mut self, field: Field) -> SubscriptionQueryBuilder {
        self.fields.push(field);
        self
    }

    pub fn build(self) -> SubscriptionQuery {
        SubscriptionQuery::new(
            self.token,
            self.home_id,
            self.fields
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>()
                .join("\\n")
                .to_string(),
        )
    }
}

impl Into<SubscriptionQuery> for SubscriptionQueryBuilder {
    fn into(self) -> SubscriptionQuery {
        self.build()
    }
}

pub enum Field {
    TimeStamp,
    Power,
    LastMeterConsumption,
    AccumulatedProduction,
    AccumulatedConsumption,
    AccumulatedConsumptionLastHour,
    AccumulatedProductionLastHour,
    AccumulatedCost,
    AccumulatedReward,
    Currency,
    MinPower,
    AveragePower,
    MaxPower,
    PowerProduction,
    PowerReactive,
    PowerProductionReactive,
    MinPowerProduction,
    MaxPowerProduction,
    LastMeterProduction,
    PowerFactor,
    VoltagePhase1,
    VoltagePhase2,
    VoltagePhase3,
    CurrentL1,
    CurrentL2,
    CurrentL3,
    SignalStrength,
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Field::TimeStamp => write!(f, "timestamp"),
            Field::Power => write!(f, "power"),
            Field::LastMeterConsumption => write!(f, "lastMeterConsumption"),
            Field::AccumulatedProduction => write!(f, "accumulatedProduction"),
            Field::AccumulatedConsumption => write!(f, "accumulatedConsumption"),
            Field::AccumulatedConsumptionLastHour => write!(f, "accumulatedConsumptionLastHour"),
            Field::AccumulatedProductionLastHour => write!(f, "accumulatedProductionLastHour"),
            Field::AccumulatedCost => write!(f, "accumulatedCost"),
            Field::AccumulatedReward => write!(f, "accumulatedReward"),
            Field::Currency => write!(f, "currency"),
            Field::MinPower => write!(f, "minPower"),
            Field::AveragePower => write!(f, "averagePower"),
            Field::MaxPower => write!(f, "maxPower"),
            Field::PowerProduction => write!(f, "powerProduction"),
            Field::PowerReactive => write!(f, "powerReactive"),
            Field::PowerProductionReactive => write!(f, "powerProductionReactive"),
            Field::MinPowerProduction => write!(f, "minPowerProduction"),
            Field::MaxPowerProduction => write!(f, "maxPowerProduction"),
            Field::LastMeterProduction => write!(f, "lastMeterProduction"),
            Field::PowerFactor => write!(f, "powerFactor"),
            Field::VoltagePhase1 => write!(f, "voltagePhase1"),
            Field::VoltagePhase2 => write!(f, "voltagePhase2"),
            Field::VoltagePhase3 => write!(f, "voltagePhase3"),
            Field::CurrentL1 => write!(f, "currentL1"),
            Field::CurrentL2 => write!(f, "currentL2"),
            Field::CurrentL3 => write!(f, "currentL3"),
            Field::SignalStrength => write!(f, "signalStrength"),
        }
    }
}
