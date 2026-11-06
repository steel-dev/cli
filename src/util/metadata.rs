use std::collections::BTreeMap;

pub fn parse_pair(raw: &str) -> Result<(String, String), String> {
    match raw.split_once('=') {
        Some((key, value)) if !key.is_empty() => Ok((key.to_string(), value.to_string())),
        _ => Err("expected KEY=VALUE".to_string()),
    }
}

pub fn collect(pairs: Vec<(String, String)>) -> anyhow::Result<BTreeMap<String, String>> {
    let mut metadata = BTreeMap::new();
    for (key, value) in pairs {
        if metadata.contains_key(&key) {
            anyhow::bail!("Metadata key \"{key}\" is given more than once.");
        }
        metadata.insert(key, value);
    }
    Ok(metadata)
}

pub fn collect_filters(pairs: Vec<(String, String)>) -> anyhow::Result<BTreeMap<String, String>> {
    let filters = collect(pairs)?;
    if let Some(key) = filters
        .iter()
        .find_map(|(key, value)| value.is_empty().then_some(key))
    {
        anyhow::bail!("Metadata filter \"{key}\" needs a value.");
    }
    Ok(filters)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair(key: &str, value: &str) -> (String, String) {
        (key.to_string(), value.to_string())
    }

    #[test]
    fn pairs_split_on_the_first_equals_sign() {
        assert_eq!(parse_pair("env=prod"), Ok(pair("env", "prod")));
        assert_eq!(parse_pair("query=a=b,c"), Ok(pair("query", "a=b,c")));
        assert_eq!(parse_pair("note="), Ok(pair("note", "")));
    }

    #[test]
    fn pairs_need_a_key_and_an_equals_sign() {
        assert!(parse_pair("env").is_err());
        assert!(parse_pair("=prod").is_err());
        assert!(parse_pair("").is_err());
    }

    #[test]
    fn repeated_keys_are_rejected() {
        let metadata = collect(vec![pair("env", "prod"), pair("run", "42")]).unwrap();
        assert_eq!(metadata.len(), 2);
        let err = collect(vec![pair("env", "prod"), pair("env", "staging")]).unwrap_err();
        assert!(err.to_string().contains("\"env\""));
    }

    #[test]
    fn filters_need_a_value_but_written_metadata_may_be_empty() {
        assert!(collect(vec![pair("note", "")]).is_ok());
        assert!(collect_filters(vec![pair("env", "prod")]).is_ok());
        let err = collect_filters(vec![pair("env", "prod"), pair("team", "")]).unwrap_err();
        assert_eq!(err.to_string(), "Metadata filter \"team\" needs a value.");
    }
}
