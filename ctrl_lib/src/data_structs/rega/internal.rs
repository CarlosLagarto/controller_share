use ctrl_prelude::domain_types::*;

/// Dimension 5
#[derive(Clone, Default)]
pub struct InternalPtr {
    /// direct pointer para o ciclo wizard no mode wizard
    pub wizard: Option<CYCLE_PTR>,
    /// para o forced sector
    pub direct: Option<CYCLE_PTR>,
    pub have_standard: bool,
}
