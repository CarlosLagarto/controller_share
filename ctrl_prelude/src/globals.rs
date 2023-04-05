pub const MICR_F: f64 = 0.000_001;
pub const NANO_F: f64 = 0.000_000_001;

pub const GIGA_F: f64 = 1_000_000_000.0;
pub const GIGA_F32: f32 = 1_000_000_000.0;

pub const GIGA_I: i64 = 1_000_000_000;

pub const GIGA_U: u64 = 1_000_000_000;

pub const PROGRAM_NAME: &str = "controller";

pub const WZRD_NAME: &str = "Wizard";
pub const MANUAL_DIRECT_SUFIX: &str = "direct";

/// Em 2022/Jun/07 sabemos que não temos mais do que 6 setores.  Quando isto mudar, recompilamos o programa.
pub const MAX_SECTORS: usize = 6;

/// para além dos 3 internos, 3 user defined....1 tipicamente será suficiente.   <br>
/// Não estou a ver use case para mais do que 2.  O 1º é o "normal". O 2º será para teste de cenas ... e o 3º já é só mesmo para ter uma folga só porque sim. <br>
/// O programa está preparado para trabalhar com N ciclos, pelo que se surgir a necessidade, é só mudar aqui as constantes
pub const MAX_INTERNALS : usize = 2; //são 2 internos 
pub const MAX_STANDARD_CYCLES: usize = 3;  //+ 3 standard, 
pub const MAX_CYCLES: usize = 5;  //dá os 5 no total.

pub const EVBR_SERVICE_THREAD: &str = "EVENT_BROKER_SERVICE"; //72 kb stack
pub const WTHR_SERVICE_THREAD: &str = "WEATHER_SERVICE"; //520 kb stack - os dados do machine learning model ocupam muito espaço
pub const MQTT_SERVICE_THREAD: &str = "MQTT_SERVICE"; //68 kb stack
pub const WBSR_SERVICE_THREAD: &str = "WEB_SERVER_SERVICE"; //4 kb stack
pub const WSS_SERVICE_THREAD: &str = "WSS_SERVER_SERVICE"; //28 kb stack
pub const DEV_SERVICE_THREAD: &str = "DEV_SERVICE"; //4 kb stack
                                                            
pub const MAIN_CTRL_THREAD: &str = "MAIN_SERVICE";

// tamanho da página no windows 32 e 64 em arquitetura x86 e x86-64
// linux também é do mesmo tamanho na mesma arquitetura.
// ainda tenho que perceber isto...porque a doc windows diz que a granularidade minima do gestor de memória são 64K
// mas para já fica assim e depois tenho que medir com a tool da intel, para ver qual a memória alocada ás threads.

// os sistemas depois podem ter configurações com páginas maiores
// mas como o meu objetivo particular aqui, é racionalizar a memória alocada em stack a cada thread,
// com o pressuposto de que é um valor pequeno (requer teste), vamos libertar o stack do cpu para o
// working set que for necessário com outras coisas

// isto não precisaria de um usize/u64, mas a signature das funções onde é utilizado a isso obriga para não andar a fazer sempre casts
pub const STACK_SIZE_UNIT: usize = 4096;


// basicamente troquei uma static global, por uma static local á thread principal.
// pelo menos o side effect é contido apenas a este módulo , e não anda espalhado pela aplicação toda.
pub static mut SHUTTING_DOWN: bool = false;//Option<SMtDeque> = None;