use thiserror::*;

#[derive(Debug, Error)]
pub enum BrokerError {
    #[error("Technical: Issue sending event msg: {0}")]
    IssueSendingMsg(String),
}

pub type EventBrokerResult<T> = Result<T, BrokerError>;
