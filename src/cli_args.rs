use ctrl_lib::app_context::start_up::*;
use ctrl_lib::app_time::ctrl_time::*;
use ctrl_lib::{log_info, logger::*};
use ctrl_prelude::{domain_types::*, string_resources::*};

/// Analisei o clap e o struct_opt.
///
/// O structopt tem algums limitações na definição do default value
///
/// Decidi avançar com o clap.
///
/// E depois implementei isto sem estas libraries, porque para 3 argumentos opcionais, não é necessário 200K de bagagem
#[inline]
pub fn handle_args(args: Vec<String>) -> StartupData {
    let mut start_date = CtrlTime::sys_time();

    let mut simulation: u8 = 0;
    let mut warp: WARP = 0;

    // let args: Vec<String> 
    let mut sd: CtrlTime = CtrlTime(0);
    let params_err = r"Parametros válidos são:

                            -d=X ,  Sets the start date and time for the simulation.  Default is the current date and time.  -d implies simulation mode
                            -w=X ,  Sets the warp factor for the simulation. Options [0,1,2,3]  Default: 0
                            -h   ,  Shows this message.";

    match args.len() {
        // sem argumentos
        2 | 3 => {
            for s in args.iter().skip(1) {
                let arg: Vec<&str> = s.split('=').collect();
                if arg.len() == 1 && arg[0] == "-h" {
                    println!("{}", params_err);
                };
                if arg.len() == 2 {
                    match arg[0] {
                        "-h" => {
                            println!("{}", params_err);
                        }
                        "-d" => {
                            if let Ok(date) = CtrlTime::try_parse_str_iso_rfc3339_to_ctrl_time(arg[1]) {
                                sd = date;
                                simulation = 1;
                            } else {
                                println!("Data no formato errado.  Têm que ser '%Y-%m-%dT%H:%M:%S%'");
                                println!("{}", params_err);
                            }
                        }
                        "-w" => {
                            let s = arg[1].parse::<u8>();
                            match s {
                                Ok(u) if u <= 3 => warp = u,
                                _ => {
                                    println!("Warp inválido.");
                                    println!("{}", params_err);
                                }
                            }
                        }
                        _ => {
                            println!("Parâmetro desconhecido: {}.  Ignorado.", arg[0]);
                            println!("{}", params_err);
                        }
                    }
                }
            }
        }
        // sem argumentos ou com mais do que 3 argumentos, ignoramos e avançamos com o default
        _ => {
            if args.len() > 3 {
                println!("Número de parâmetros inválido.  A ignorar e a continuar com os valores default.");
            }
        }
    }

    if simulation == 1 && sd!= CtrlTime(0){
        start_date = sd;
    }

    setup_time_simulator(start_date, simulation, warp);

    log_info!(info_starting_controller(simulation, &start_date.as_rfc3339_str_e(), warp));

    StartupData::build_all(start_date, simulation, warp)
}

#[inline]
pub fn setup_time_simulator(start_date: CtrlTime, simulation: u8, warp: WARP) {
    //isto é no arranque e só está esta thread a correr pelo que podemos dar liberdade ao compilador para reordenar as instruções como quiser.
    //este unsafe é seguro porque só aqui é que é alterado este valor
    unsafe {
        STARTUP_TIME = start_date.0;
        REAL_START = CtrlTime::sys_time().0;
        SIM_BOOL = simulation != 0;
        SIM_U8 = simulation;
        WARP = warp;
    }
}

#[cfg(test)]
mod tests {
    use ctrl_lib::app_time::ctrl_time::CtrlTime;

    use crate::cli_args::setup_time_simulator;

    #[test]
    fn test_parse_dates() {
        let time = CtrlTime::try_parse_str_iso_rfc3339_to_ctrl_time("2022-01-28T12:23:14");
        if let Ok(t) = time {
            println!("data:{}, result: {:?}", t, t.as_utc_date_time_e());
        }
    }

    #[test]
    fn test_setup_time_simulator() {
        use ctrl_lib::utils::elapsed_dyn;
        use std::time::Instant;

        let t = Instant::now();
        setup_time_simulator(CtrlTime::sys_time(), 0, 0);
        println!("Tempo setup: {}", elapsed_dyn(t.elapsed().as_nanos() as u64));
    }
}
