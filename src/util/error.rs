use serde_json::{Value, json};

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct InvalidInput(pub String);

pub fn invalid_input(message: impl std::fmt::Display) -> anyhow::Error {
    InvalidInput(message.to_string()).into()
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ApiBadResponse(pub &'static str);

#[derive(Debug, thiserror::Error)]
#[error("Could not read {0}")]
pub struct LocalStateInvalid(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginRequired {
    pub verification_uri: String,
    pub verification_uri_complete: String,
    pub user_code: String,
    pub expires_at: u64,
    pub started: bool,
}

impl LoginRequired {
    pub fn expires_in(&self) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.expires_at.saturating_sub(now)
    }

    pub fn to_json(&self) -> Value {
        let mut login = json!({
            "status": "pending",
            "verificationUrl": self.verification_uri_complete,
            "verificationUri": self.verification_uri,
            "userCode": self.user_code,
            "expiresInSeconds": self.expires_in(),
            "nextAction": "Open verificationUrl in a browser and approve the code, then run the same command again.",
        });
        if let Ok(at) =
            jiff::Timestamp::from_second(i64::try_from(self.expires_at).unwrap_or(i64::MAX))
        {
            login["expiresAt"] = json!(at.to_string());
        }
        json!({ "login": login })
    }
}

fn minutes(seconds: u64) -> String {
    match seconds.div_ceil(60) {
        0 | 1 => "1 minute".to_string(),
        n => format!("{n} minutes"),
    }
}

impl std::fmt::Display for LoginRequired {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Not signed in to Steel. Open {} in a browser and approve code {} (expires in {}), then run this command again.",
            self.verification_uri_complete,
            self.user_code,
            minutes(self.expires_in())
        )
    }
}

impl std::error::Error for LoginRequired {}

#[cfg(test)]
mod tests {
    use super::*;

    fn handoff(expires_at: u64) -> LoginRequired {
        LoginRequired {
            verification_uri: "https://auth.steel.dev/device".into(),
            verification_uri_complete: "https://auth.steel.dev/device?user_code=ABCD2345".into(),
            user_code: "ABCD2345".into(),
            expires_at,
            started: true,
        }
    }

    #[test]
    fn login_required_names_the_url_code_and_next_step() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let message = handoff(now + 600).to_string();
        assert!(message.contains("https://auth.steel.dev/device?user_code=ABCD2345"));
        assert!(message.contains("ABCD2345"));
        assert!(message.contains("10 minutes"));
        assert!(message.contains("run this command again"));
    }

    #[test]
    fn login_required_json_carries_the_handoff() {
        let data = handoff(1_900_000_000).to_json();
        assert_eq!(data["login"]["status"], "pending");
        assert_eq!(
            data["login"]["verificationUrl"],
            "https://auth.steel.dev/device?user_code=ABCD2345"
        );
        assert_eq!(data["login"]["userCode"], "ABCD2345");
        assert_eq!(data["login"]["expiresAt"], "2030-03-17T17:46:40Z");
    }

    #[test]
    fn expired_handoff_reports_zero_seconds_left() {
        assert_eq!(handoff(1).expires_in(), 0);
        assert_eq!(minutes(0), "1 minute");
        assert_eq!(minutes(61), "2 minutes");
    }
}
