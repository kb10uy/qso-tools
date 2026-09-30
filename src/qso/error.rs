use std::num::ParseFloatError;

use thiserror::Error as ThisError;
use time::error::Parse as TimeParseError;

use crate::qso::band::InvalidBand;

#[derive(Debug, ThisError)]
pub enum QsoError {
    #[error("missing ADIF field: {0}")]
    MissingAdifField(&'static str),

    #[error("datetime parse error: {0}")]
    DateTimeParse(#[from] TimeParseError),

    #[error("band parse error")]
    BandParse(#[from] InvalidBand),

    #[error("frequency parse error: {0}")]
    FrequencyParse(#[from] ParseFloatError),

    #[error("QSL status parse error: {0}")]
    QslParse(String),
}
