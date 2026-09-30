use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Instrument {
    pub rig: String,
    pub antenna: String,
    pub default_power: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Park {
    pub name_en: Option<String>,
    pub name_ja: Option<String>,
}
