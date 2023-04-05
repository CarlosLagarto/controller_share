use crate::data_structs::client::sector_cli::*;
use crate::data_structs::client::sync_op::*;
use crate::services::electronics::valve_state::*;
use crate::{app_time::ctrl_time::*, db::*};
use ctrl_prelude::domain_types::*;

/// Dimension 96
#[derive(Clone, Debug, Default)]
pub struct Sector {
    pub desc: String, //+ 15?
    pub name: String, // + 11
    pub last_watered_in: CtrlTime,
    pub last_change: CtrlTime,
    pub deficit: f32,
    pub percolation: f32,  // mm/hora
    pub debit: f32,        // mm/minuto
    pub max_duration: f32, // minutos
    pub stress_perc: f32,
    pub stress_score: u8,

    pub id: SECTOR_ID,
    pub enabled: bool,
    pub op: SyncOp,

    pub update_db: bool,
    pub state: RelayState,
    pub device_id: DEVICE_ID,
}

impl core::convert::From<&SqlRow<'_>> for Sector {
    #[inline]
    fn from(sql_row: &SqlRow) -> Sector {
        // let x = u16::MAX;
        let s_op : String= sql_row.get(10).unwrap();
        Sector {
            id: sql_row.get(0).unwrap(),
            desc: sql_row.get(1).unwrap(),
            deficit: sql_row.get(2).unwrap(),
            percolation: sql_row.get(3).unwrap(),
            debit: sql_row.get(4).unwrap(),
            last_watered_in: CtrlTime::from_ux_ts(sql_row.get::<usize, UTC_UNIX_TIME>(5).unwrap()),
            enabled: sql_row.get(6).unwrap(),
            max_duration: sql_row.get(7).unwrap(),
            name: sql_row.get(8).unwrap(),
            last_change: CtrlTime::from_ux_ts(sql_row.get::<usize, UTC_UNIX_TIME>(9).unwrap()),
            op: SyncOp::from_str(&s_op),
            device_id: sql_row.get(11).unwrap(),
            ..Default::default()
        }
    }
}

impl Sector {
    /// Por definição, na ativação manual, ao fim de 12 horas desliga-se o setor.
    /// É muito pouco provável que se precise mais do que 12 horas a regar algo....
    /// Na eventualidade de ser preciso, deve-se criar um ciclo Direct para isso ser explicito
    pub const MAX_SECTOR_WORK_MINUTES: f32 = 720.;

    #[inline]
    pub fn update_with_cli_info(&mut self, c_s: &SectorCli) {
        // aqui não há novos setores nem se apaga, porque isso é controlado apenas por configuração no servidor.
        // a informação das runs de arranque e paragem também é automatico e só no servidor
        self.name = c_s.name.clone();
        self.deficit = c_s.deficit;
        self.enabled = c_s.enabled;
        self.max_duration = c_s.max_duration;
        self.percolation = c_s.percolation; //  # mm / minuto
        self.debit = c_s.debit; // # mm / minuto
        self.op = SyncOp::U;
        // só atualizamos se a alteração do cliente é posterior á alteração no servidor.  Isto tem potencial para criar ambiguidades, mas vamos avançar para já asim
        let last_change = CtrlTime::from_ux_ts(c_s.last_change);
        if last_change > self.last_change{ 
            self.last_change = last_change;
        }
    }
}
