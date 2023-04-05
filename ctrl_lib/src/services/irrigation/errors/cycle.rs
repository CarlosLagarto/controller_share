use thiserror::*;

#[derive(Debug, Error)]
pub enum CycleError {
    #[error("Error setting up wizard cycle schedule.")]
    CantSetupWizardSchedule,
    #[error("Error setting up wizard compensation cycle schedule.")]
    CantSetupWizardCompensationSchedule,
    #[error("Error deleting cycle schedule: {0}")]
    CantDeleteCycleSchedule(String),
    #[error("Error updating cycle schedule: {0}")]
    CantUpdateCycleSchedule(String),
}

pub type CycleResult<T> = Result<T, CycleError>;
