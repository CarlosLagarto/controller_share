use num_enum::UnsafeFromPrimitive;
///
///     Rain = 0, <br>
///     Temp = 1,<br>
///     Humidity = 2,<br>
///     WindSpeed = 3,<br>
///     WindBearing = 4,<br>
///     Pressure = 5,<br>
///     SolarRadiation = 6,<br>
///     WaterPumpCurrentDetection = 7, <br>
///     DewPoint = 8 <br>
///     WattHora = 9,
/// 
// Convêm não mexer nos ids para manter a coerência 
// existe, boas práticas conflituantes
// por um lado, o design e análise deve ser genérico, portanto abstrair dos detalhes
// por outro, a performance requer coisas concretas, e tem que se pesar a boa prática de - "isto não vai evoluir, e quando evoluir, logo se evolui o design".
// portanto, para já está assim
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, Copy, PartialEq, UnsafeFromPrimitive)]
#[repr(u8)]
pub enum Sensor {
    Rain = 0,
    TempOutside = 1,
    HrOutside = 2,
    WindSpeed = 3,
    WindBearing = 4,
    Pressure = 5,
    SolarRadiation = 6,
    WaterPumpCurrentDetection = 7,
    DewPoint = 8,
    WattHora = 9,
}

pub const MAX_SENSORS: usize = 10;