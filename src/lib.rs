pub mod models;

use base64::prelude::*;
use models::Info;
use url::Url;

#[derive(Debug)]
pub struct SimpleFinBridge {
    client: reqwest::blocking::Client,
    url: Url,
}

impl SimpleFinBridge {
    pub fn new(token: &str) -> Self {
        let claim_url = String::from_utf8(BASE64_STANDARD.decode(token).unwrap()).unwrap();
        let client = reqwest::blocking::Client::new();
        let access_url = client.post(claim_url).send().unwrap().text().unwrap();

        let parsed_url = Url::parse(&access_url).unwrap();

        Self {
            client,
            url: parsed_url.clone(),
        }
    }

    pub fn info(&self) -> Info {
        let mut info_url = self.url.clone();
        info_url.set_path(&format!("{}/info", info_url.path()));

        self.client.get(info_url).send().unwrap().json().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_TOKEN: &str =
        "aHR0cHM6Ly9iZXRhLWJyaWRnZS5zaW1wbGVmaW4ub3JnL3NpbXBsZWZpbi9jbGFpbS9ERU1P";

    #[test]
    fn info() {
        let simplefin_bridge = SimpleFinBridge::new(TEST_TOKEN);

        assert_eq!(
            simplefin_bridge.info(),
            Info {
                versions: vec![String::from("1.0")]
            }
        )
    }
}
