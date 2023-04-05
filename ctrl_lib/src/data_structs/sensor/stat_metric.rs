// Tive aqui alguma indecisão sobre quais as métricas diárias a registar
// por um lado há a tentação de calcular-se tudo e registar-se.
// por outro, isso fica a ocupar espaço na BD para um objetivo ainda não bem identificado nesta fase do programa
// onde acresce que a raw data para o calculo se/quando for necessário também está na BD, o que me levou a inclinar a manter os basicos:
//
// Como percebi que tinha que processar todos os registos para criar as métircas diárias, pareceu-me idiota ir fazer isso tudo outra vez 
// sempre que se corria o algoritmo de predição, pelo que fica mesmo tudo na base de dados
//
// É importante ter na cabeça a referência que isto é calculado no inicio do dia, portanto a eT e o SumRain são das 24 horas anteriores
// e a probabilidade é para o dia que está a começar
//
use num_enum::UnsafeFromPrimitive;

/// Dimension 1
/// <br>
/// AvgHumidity         = 0,  //necessário também para o cálculo do eT <br>
//  AvgPressure         = 1,  //necessário também para o cálculo do eT <br>
/// AvgWindSpeed        = 2, //necessário também para o cálculo do eT <br>
/// MaxHumidity         = 3,  //necessário também para o cálculo do eT <br>
/// MaxTemp             = 4,      //necessário também para o cálculo do eT <br>
/// MinHumidity         = 5,  //necessário também para o cálculo do eT <br>
/// MinTemp             = 6,      //necessário também para o cálculo do eT <br>
/// SumRain             = 7,       <br>
/// EvapoTranspiration  = 8, // este não é preciso para o model ML, mas é preciso para o weather service <br>
/// AvgTemp             = 9, <br>
/// MaxPressure         = 10, <br>
/// MinPressure         = 11, <br>
/// MaxWindSpeed        = 12, <br>
/// MinWindSpeed        = 13, <br>
/// SolarRadiation      = 14, <br>
/// AvgWindDirection    = 15, <br>
/// TempAt0             = 16, <br>
/// TempAt6         = 17, <br>
/// TempAt12 = 18, <br>
/// TempAt18 = 19, <br>
/// PressAt0 = 20, <br>
/// PressAt6 = 21, <br>
/// PressAt12 = 22, <br>
/// PressAt18 = 23, <br>
/// HrAt0 = 24, <br>
/// HrAt6 = 25, <br>
//// HrAt12 = 26, <br>
/// HrAt18 = 27, <br>
/// WsAt0 = 28, <br>
/// WsAt6 = 29, <br>
/// WsAt12 = 30, <br>
/// WsAt18 = 31, <br>
/// WdAt0 = 32, <br>
/// WdAt6 = 33, <br>
/// WdAt12 = 34, <br>
/// WdAt18 = 35, <br>
/// AvgDwp = 36, <br>
/// MaxDwp = 37, <br>
/// MinDwp = 38, <br>
/// DwpAt0 = 39, <br>
/// DwpAt6 = 40, <br>
/// DwpAt12 = 41, <br>
/// DwpAt18 = 42, <br>
/// PressureDwpRatio = 43, <br>
/// HumidityGtERatio = 44, <br>
/// RainClass = 45, <br>
/// RainClassForecast = 46, <br>
/// DayNr = 47, // Isto não é uma métrica, mas é uma martelada para não criar outro enum só para o modelo de machine learning <br>
/// RainProbability = 48,  // este não é preciso para o model ML, mas é preciso para o weather service <br>
#[allow(clippy::derive_partial_eq_without_eq)]
#[repr(u8)]
#[derive(PartialEq, UnsafeFromPrimitive)]
pub enum Metric {
    AvgHumidity = 0,  //necessário também para o cálculo do eT
    AvgPressure = 1,  //necessário também para o cálculo do eT
    AvgWindSpeed = 2, //necessário também para o cálculo do eT
    MaxHumidity = 3,  //necessário também para o cálculo do eT
    MaxTemp = 4,      //necessário também para o cálculo do eT
    MinHumidity = 5,  //necessário também para o cálculo do eT
    MinTemp = 6,      //necessário também para o cálculo do eT   
    SumRain = 7,      
    EvapoTranspiration = 8, // este não é preciso para o model ML, mas é preciso para o weather service
    AvgTemp = 9,
    MaxPressure = 10,
    MinPressure = 11,
    MaxWindSpeed = 12,
    MinWindSpeed = 13,
    SolarRadiation = 14,
    AvgWindDirection = 15,
    TempAt0 = 16,
    TempAt6 = 17,
    TempAt12 = 18,
    TempAt18 = 19,
    PressAt0 = 20,
    PressAt6 = 21,
    PressAt12 = 22,
    PressAt18 = 23,
    HrAt0 = 24,
    HrAt6 = 25,
    HrAt12 = 26,
    HrAt18 = 27,
    WsAt0 = 28,
    WsAt6 = 29,
    WsAt12 = 30,
    WsAt18 = 31,
    WdAt0 = 32,
    WdAt6 = 33,
    WdAt12 = 34,
    WdAt18 = 35,
    AvgDwp = 36,
    MaxDwp = 37,
    MinDwp = 38,
    DwpAt0 = 39,
    DwpAt6 = 40,
    DwpAt12 = 41,
    DwpAt18 = 42,
    PressureDwpRatio = 43,
    HumidityGtERatio = 44,
    RainClass = 45,
    RainClassForecast = 46,
    DayNr = 47, // Isto não é uma métrica, mas é uma martelada para não criar outro enum só para o modelo de machine learning
    RainProbability = 48,  // este não é preciso para o model ML, mas é preciso para o weather service
}

pub const MAX_FEATURES: usize = 49;

pub const NR_CLASSES: usize = 5;
pub const CLASSES: [u8; NR_CLASSES] = [0, 1, 2, 3, 4];
pub const MAX_DAILY_ROWS: usize = 1825; // vai ser tipicamente inferior, mas tenho que quantificar e analisar os dados.
