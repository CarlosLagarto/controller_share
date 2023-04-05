use serde::{Deserialize, Serialize};

use ctrl_prelude::domain_types::*;

/// Esta é a estrutura para trabalhar internamente no programa <br>
/// <br>
/// cycle = ao id para a collection dos ciclos <br>
/// sector = ao id para a colection dos setores <br>
/// run_sector = ao id para a colection dos run sectors (é um ptr que não é necessariamente igual ao id do setor, porque podem haver setores que são skipped) <br>
/// <br>
/// Dimension = 6
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct RunningPtr {
    /// cycle = ao id para a collection dos ciclos <br>
    pub cycle: Option<CYCLE_PTR>,
    /// sector = ao id para a colection dos setores <br>
    pub sec_id: Option<SECTOR_ID>,
    /// run_sector = ao id para a colection dos run sectors 
    pub run_sec_ptr: Option<SECTOR_PTR>,
}

impl RunningPtr{
    #[inline]
    pub fn reset_sec(&mut self){
        self.sec_id = None;
        self.run_sec_ptr= None;
    }
    #[inline]
    pub fn reset_all(&mut self){
        self.reset_sec();
        self.cycle = None;
    }
}
impl RunningPtr {
    #[inline]
    pub const fn new(cycle: Option<CYCLE_PTR>, sector: Option<SECTOR_ID>, run_sector: Option<SECTOR_PTR>) -> Self {
        Self { cycle, sec_id: sector, run_sec_ptr: run_sector }
    }
}
