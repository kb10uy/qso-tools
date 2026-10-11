use std::{convert::Infallible, str::FromStr};

/// Script argument given as `KEY` or `KEY=VALUE`; a bare `KEY` has an empty value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScriptArg {
    pub key: String,
    pub value: String,
}

impl FromStr for ScriptArg {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (key, value) = s.split_once('=').unwrap_or((s, ""));
        Ok(ScriptArg {
            key: key.to_string(),
            value: value.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_script_arg() {
        let arg: ScriptArg = "key=value".parse().unwrap();
        assert_eq!(arg.key, "key");
        assert_eq!(arg.value, "value");

        let arg: ScriptArg = "a=b=c".parse().unwrap();
        assert_eq!(arg.key, "a");
        assert_eq!(arg.value, "b=c");

        let arg: ScriptArg = "flag".parse().unwrap();
        assert_eq!(arg.key, "flag");
        assert_eq!(arg.value, "");
    }
}
