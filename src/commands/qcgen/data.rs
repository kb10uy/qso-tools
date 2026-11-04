use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Park {
    pub name_en: Option<String>,
    pub name_ja: Option<String>,
}
