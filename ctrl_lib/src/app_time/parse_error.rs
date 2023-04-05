use thiserror::*;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Data não reconhecida: {0}.  O formato aceite é: 'YYYY-mm-ddTHH:MM:SS'")]
    UnknownDate(String),
    #[error("By design, só está preparada e testada para funcionar entre 1970 e 2077")]
    InvalidYear,
    #[error("Mês inválido")]
    InvalidMonth,
    #[error("Dia Inválido.  O mês {0} só têm {1} dias")]
    InvalidDay(u8, u8),
    #[error("Hora inválida.  A hora deve ser entre 0 e 23")]
    InvalidHour,
    #[error("Minutos inválidos.  Os minutos são entre 0 e 59")]
    InvalidMinutes,
    #[error("Segundos inválidos. Os segundos são entre 0 e 59")]
    InvalidSeconds,
}

pub type ParseDateResult<T> = std::result::Result<T, ParseError>;
