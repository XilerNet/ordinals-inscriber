#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogTypes {
    PaymentRequested,
    PaymentReceivedUnconfirmed,
    PaymentReceivedConfirmed,
}

impl From<&str> for LogTypes {
    fn from(value: &str) -> Self {
        match value {
            "payment_requested" => LogTypes::PaymentRequested,
            "payment_received_unconfirmed" => LogTypes::PaymentReceivedUnconfirmed,
            "payment_received_confirmed" => LogTypes::PaymentReceivedConfirmed,
            _ => panic!("Invalid log type"),
        }
    }
}

impl Into<&str> for LogTypes {
    fn into(self) -> &'static str {
        match self {
            LogTypes::PaymentRequested => "payment_requested",
            LogTypes::PaymentReceivedUnconfirmed => "payment_received_unconfirmed",
            LogTypes::PaymentReceivedConfirmed => "payment_received_confirmed",
        }
    }
}

impl Into<String> for LogTypes {
    fn into(self) -> String {
        let s: &'static str = self.into();
        s.to_string()
    }
}
