// Fiz testes mas a comparação não foi conclusiva.
// Basicamente não consegui assegurar a mesma baseline de comparação, porque o spdlog não faz o flush em modo assincrono bem.  
// Ficam sempre coisas nos buffers e perdem-se mensagens.
// neste contexto o que "parece" é que em condições identicas o flexi-logger é mais rápido por um fator de 10.
// como o spdlog "come" mensagens, ficam coisas por escrever, e por isso em certas circunstancias parece mais rapido.
// O "parece" é suficiente para já.  Não justifica investir tempo em algo que ainda nao está estavel .  
// Falei com o autor da lib, e a parte assincrona ainda está em desenvolvimento.
// Desta forma avança-se com o que está estavel e a funcionar, e que á data, até é o mais rápido.
// Outra decisão é que parte da lentidão e consumo de recursos é duplicar o log para o file e para o stdout
// mandando só para o file, tira-se mais de metade do tempo, e o objetivo final, que é ter logo, é á mesma assegurado.

use std::time::Instant;

use ctrl_lib::log_info;
use ctrl_lib::logger::info;
use ctrl_lib::utils::elapsed_dyn;

#[test]
fn test_log() {

    let mut t0: Instant;
    let mut total: u64;

    // let _logger_handle = logger::initialize_logger(AppLog::new());

    total = 0;
    for _i in 0..50 {
        t0 = Instant::now();
        log_info!("loging a somewhat long long long long long long long long long long long long long long long long long long long string ");
        total += t0.elapsed().as_nanos() as u64;
    }
    println!("flexi--logger: {}", elapsed_dyn(total / 50));
}
