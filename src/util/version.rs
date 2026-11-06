use std::cmp::Ordering;

pub fn is_newer(candidate: &str, current: &str) -> bool {
    compare(candidate, current) == Ordering::Greater
}

fn compare(a: &str, b: &str) -> Ordering {
    let (a_core, a_pre) = split(a);
    let (b_core, b_pre) = split(b);
    a_core
        .cmp(&b_core)
        .then_with(|| compare_prerelease(a_pre, b_pre))
}

fn split(version: &str) -> (Vec<u64>, Option<&str>) {
    let version = version.trim();
    let version = version.strip_prefix('v').unwrap_or(version);
    let version = version.split_once('+').map_or(version, |(base, _)| base);
    let (core, pre) = match version.split_once('-') {
        Some((core, pre)) => (core, Some(pre)),
        None => (version, None),
    };
    let mut numbers: Vec<u64> = core
        .split('.')
        .map(|part| part.parse().unwrap_or(0))
        .collect();
    while numbers.len() < 3 {
        numbers.push(0);
    }
    (numbers, pre)
}

fn compare_prerelease(a: Option<&str>, b: Option<&str>) -> Ordering {
    let (a, b) = match (a, b) {
        (None, None) => return Ordering::Equal,
        (None, Some(_)) => return Ordering::Greater,
        (Some(_), None) => return Ordering::Less,
        (Some(a), Some(b)) => (a, b),
    };
    let mut left = a.split('.');
    let mut right = b.split('.');
    loop {
        let order = match (left.next(), right.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) => match (x.parse::<u64>(), y.parse::<u64>()) {
                (Ok(x), Ok(y)) => x.cmp(&y),
                (Ok(_), Err(_)) => Ordering::Less,
                (Err(_), Ok(_)) => Ordering::Greater,
                (Err(_), Err(_)) => x.cmp(y),
            },
        };
        if order != Ordering::Equal {
            return order;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_numbers() {
        assert!(is_newer("0.3.1", "0.3.0"));
        assert!(is_newer("0.4.0", "0.3.9"));
        assert!(is_newer("1.0.0", "0.99.99"));
        assert!(is_newer("0.10.0", "0.9.1"));
        assert!(is_newer("v1.0.0", "0.9.9"));
        assert!(!is_newer("0.3.0", "0.3.0"));
        assert!(!is_newer("0.2.0", "0.3.0"));
    }

    #[test]
    fn a_release_is_newer_than_its_previews() {
        assert!(is_newer("0.5.0", "0.5.0-preview.6"));
        assert!(!is_newer("0.5.0-preview.6", "0.5.0"));
        assert!(!is_newer("0.4.2", "0.5.0-preview.6"));
        assert!(is_newer("0.5.1", "0.5.0-preview.6"));
    }

    #[test]
    fn previews_compare_by_identifier() {
        assert!(is_newer("0.5.0-preview.7", "0.5.0-preview.6"));
        assert!(is_newer("0.5.0-preview.10", "0.5.0-preview.9"));
        assert!(is_newer("0.5.0-rc.1", "0.5.0-preview.9"));
        assert!(is_newer("0.5.0-preview.1.1", "0.5.0-preview.1"));
        assert!(!is_newer("0.5.0-preview.6", "0.5.0-preview.6"));
    }
}
