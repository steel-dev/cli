use std::collections::BTreeMap;

use anyhow::{Result, bail};

pub fn parse_pairs(flag: &str, shape: &str, pairs: &[String]) -> Result<BTreeMap<String, String>> {
    let mut parsed = BTreeMap::new();
    for pair in pairs {
        let Some((name, value)) = pair.split_once('=') else {
            bail!("{flag} wants {shape}, got {pair:?}.");
        };
        if name.is_empty() {
            bail!("{flag} wants a name before the '=', got {pair:?}.");
        }
        if parsed.insert(name.to_string(), value.to_string()).is_some() {
            bail!("{flag} {name} was given twice.");
        }
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_pairs_and_refuses_bad_ones() {
        assert!(parse_pairs("--env", "KEY=VALUE", &[]).unwrap().is_empty());
        let parsed = parse_pairs(
            "--env",
            "KEY=VALUE",
            &["A=1".into(), "B=x=y".into(), "C=".into()],
        )
        .unwrap();
        assert_eq!(parsed["A"], "1");
        assert_eq!(parsed["B"], "x=y");
        assert_eq!(parsed["C"], "");
        let err = parse_pairs("--env", "KEY=VALUE", &["NOPE".into()]).unwrap_err();
        assert!(err.to_string().contains("--env wants KEY=VALUE"));
        assert!(parse_pairs("--env", "KEY=VALUE", &["=1".into()]).is_err());
        let err =
            parse_pairs("--secret", "NAME=SECRET_ID", &["A=1".into(), "A=2".into()]).unwrap_err();
        assert!(err.to_string().contains("--secret A was given twice"));
    }
}
