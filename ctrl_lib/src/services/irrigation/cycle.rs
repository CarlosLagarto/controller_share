use crate::app_time::{ctrl_time::*, schedule::*, schedule_params::*};
use crate::config::geo_pos::*;
use crate::data_structs::{client::sync_op::*, rega::mode::*, rega::watering_status::*};
use crate::services::irrigation::{cycle_run::*,cycle_type::*};
use crate::{db::*, string_concat::*};
use ctrl_prelude::{domain_types::*, globals::*};

/// Dimension = 112
#[derive(Clone, Debug, Default)]
pub struct Cycle {
    pub run: CycleRun,
    pub schedule: Schedule,
    pub name: String, //? 17? m
    pub last_change: CtrlTime,
    pub last_run: CtrlTime,
    pub op: SyncOp,
    pub sunrise_flg: SUN_FLAG,
    pub sunset_flg: SUN_FLAG,
    pub ptr: Option<CYCLE_PTR>,
    pub cycle_type: CycleType,
}

impl Cycle {
    #[inline]
    #[rustfmt::skip]
    pub fn new(schedule: Schedule, name: String, sunrise_flg: SUN_FLAG, time: CtrlTime, cycle_type: CycleType) -> Self {
        Self {
            name,
            sunrise_flg,
            last_change: time,
            run: CycleRun { status: WateringStatus::Waiting, ..Default::default() },
            schedule,
            cycle_type,
            ..Default::default()
        }
    }

    #[inline]
    pub fn update_with_cli_info(&mut self, cli_cycle: &Cycle) {
        self.name = cli_cycle.name.clone();
        self.op = cli_cycle.op.clone();
        self.sunrise_flg = cli_cycle.sunrise_flg;
        self.sunset_flg = cli_cycle.sunset_flg;
        self.last_change = cli_cycle.last_change;
        self.run.end = cli_cycle.run.end;
        self.last_run = cli_cycle.last_run;
        self.schedule = cli_cycle.schedule.clone();
        self.cycle_type = CycleType::Standard; //do cliente só podem vir ciclos standard
    }

    // #[inline]
    // pub fn set_start(&mut self, start: CtrlTime) {
    //     self.run.start = start;
    //     self.schedule.start = start;
    // }

    #[inline]
    #[rustfmt::skip]
    pub fn new_wizard(time: CtrlTime, geo_pos: GeoPos) -> Self {
        // calcula o inicio do ciclo de forma a acabar antes do nascer do sol
        // para melhorar a eficiencia da rega(perdas devido á evaporação e vento durante a rega)
        let mut start = adjust_start_date_to_sunrise(&geo_pos, time);
            // se o inicio previsto já foi - passa para o dia seguinte - como falamos de 1 dia, a diff. para a hora é pequena e desprezada
        if start < time { start = start.add_days(1); } 
        // Após leitura e alguma reflexão, regar de 2 em 2 dias não é suficiente para muitas situações.
        // Por outro, dificultava todas as contas e testes para calcular o valor diario, especialmente em situações agrestes e no verão.  
        //
        // Portanto vamos simplificar, e regar diariamente de acordo com a formula do wizard, e simplesmente saltar o dia ou os setores quando as condições o justificarem, 
        // ou se começar a chover e aguardamos que pare para recalcular o novo tempo de rega em função das novas condições.
        //
        // TDD - Test Driven Development é cada vez melhor.  Se não conseguimos testar algo, é mesmo porque ou o design, ou a implementação não estão de forma
        // a que se consiga ser eficiente e ter a certeza que a coisa está bem feita.  A reorganização para conseguir testar tem tido o side effect de que
        // o código fica com uma arquitetura melhor.
        let schedule = Schedule::build_run_forever(start, 1, ScheduleRepeatUnit::Days);
        
        Cycle::new(schedule, string_concat!(WZRD_NAME, "-auto"), 1, time, CycleType::Wizard)
    }

    #[inline]
    pub fn new_direct(start: CtrlTime) -> Self {
        let schedule = Schedule::build_run_once(start);
        Cycle::new(schedule, MANUAL_DIRECT_SUFIX.to_owned(), 0, start, CycleType::Direct)
    }

    #[inline]
    pub fn get_next_event(&self, mode: Mode, after: CtrlTime, geo_pos: &GeoPos) -> Result<Option<CtrlTime>, ScheduleError> {
        let mut _next_start_ts = find_next_event(after, &self.schedule)?;
        match _next_start_ts {
            Some(time) if self.sunrise_flg == 1 && mode.is_wizard() => {
                // vai ajustar o inicio para o nascer do sol
                let next_ts_sunrise_corr = adjust_start_date_to_sunrise(geo_pos, time);
                _next_start_ts = Some(next_ts_sunrise_corr);
                if next_ts_sunrise_corr < after {
                    // se o nascer do sol é antes do tempo definido (after), o que fazemos?
                    // Isto só poderá acontecer no primeiro dia a seguir á definição, ou quando há erros ou interrupções.
                    //
                    _next_start_ts = Some(next_ts_sunrise_corr.add_days(1));
                }
            }
            _ => (),
        }
        Ok(_next_start_ts)
    }
}

impl From<&SqlRow<'_>> for Cycle {
    #[inline]
    fn from(sql_row: &SqlRow) -> Cycle {
        let sql_row = sql_row;
        let op: String = sql_row.get(17).unwrap();
        Cycle {
            ptr: None,
            run: CycleRun {
                cycle_id: sql_row.get(0).unwrap(),
                run_id: sql_row.get(3).unwrap(),
                status: unsafe { WateringStatus::from_unchecked(sql_row.get(2).unwrap()) },
                start: CtrlTime::from_ux_ts(sql_row.get(4).unwrap()),
                ..Default::default()
            },
            name: sql_row.get(1).unwrap(),
            last_change: CtrlTime::from_ux_ts(sql_row.get(16).unwrap()),
            op: SyncOp::from_str(&op),
            // sim: sql_row.get(18).unwrap(),
            last_run: CtrlTime::from_ux_ts(sql_row.get(5).unwrap()),
            sunrise_flg: sql_row.get(6).unwrap(),
            sunset_flg: sql_row.get(7).unwrap(),
            schedule: Schedule {
                start: CtrlTime::from_ux_ts(sql_row.get(4).unwrap()),
                repeat_kind: unsafe { ScheduleRepeat::from_unchecked(sql_row.get(8).unwrap()) },
                repeat_spec_wd: sql_row.get(9).unwrap(),
                repeat_every_qty: sql_row.get(10).unwrap(),
                repeat_every_unit: unsafe { ScheduleRepeatUnit::from_unchecked(sql_row.get(11).unwrap()) },
                stop_condition: unsafe { ScheduleStop::from_unchecked(sql_row.get(12).unwrap()) },
                stop_retries: sql_row.get(13).unwrap(),
                stop_date_ts: CtrlTime::from_ux_ts(sql_row.get(14).unwrap()),
                retries_count: sql_row.get(15).unwrap(),
            },
            cycle_type: unsafe { CycleType::from_unchecked(sql_row.get(18).unwrap()) },
        }
    }
}
