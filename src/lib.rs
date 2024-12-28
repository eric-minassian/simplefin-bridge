pub mod models;

use base64::prelude::*;
use models::{AccountSet, Info};
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

    pub fn accounts(&self, params: Option<AccountsParams>) -> AccountSet {
        let mut accounts_url = self.url.clone();
        accounts_url.set_path(&format!("{}/accounts", accounts_url.path()));

        if let Some(params) = params {
            let query = params.to_query_string();
            accounts_url.set_query(Some(&query));
        }

        self.client
            .get(accounts_url)
            .send()
            .unwrap()
            .json()
            .unwrap()
    }
}

pub struct AccountsParams {
    pub start_date: Option<i64>,
    pub end_date: Option<i64>,
    pub pending: Option<bool>,
    pub account_ids: Option<Vec<String>>,
    pub balances_only: Option<bool>,
}

impl AccountsParams {
    fn to_query_string(&self) -> String {
        let mut params = Vec::new();

        if let Some(start_date) = self.start_date {
            params.push(format!("start-date={}", start_date));
        }
        if let Some(end_date) = self.end_date {
            params.push(format!("end-date={}", end_date));
        }
        if let Some(pending) = self.pending {
            if pending {
                params.push("pending=1".to_string());
            }
        }
        if let Some(account_ids) = &self.account_ids {
            for id in account_ids {
                params.push(format!("account={}", id));
            }
        }
        if let Some(balances_only) = self.balances_only {
            if balances_only {
                params.push("balances-only=1".to_string());
            }
        }

        params.join("&")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    const TEST_TOKEN: &str =
        "aHR0cHM6Ly9iZXRhLWJyaWRnZS5zaW1wbGVmaW4ub3JnL3NpbXBsZWZpbi9jbGFpbS9ERU1P";

    fn timestamp(year: i32, month: u32, day: u32) -> i64 {
        Utc.with_ymd_and_hms(year, month, day, 0, 0, 0)
            .unwrap()
            .timestamp()
    }

    fn setup_bridge() -> SimpleFinBridge {
        SimpleFinBridge::new(TEST_TOKEN)
    }

    #[test]
    fn test_info() {
        let bridge = setup_bridge();
        assert_eq!(
            bridge.info(),
            Info {
                versions: vec![String::from("1.0")]
            }
        );
    }

    #[test]
    fn test_accounts_no_params() {
        let bridge = setup_bridge();
        let accounts = bridge.accounts(None);
        assert!(accounts.errors.is_empty());
        assert!(!accounts.accounts.is_empty());

        // Verify account structure
        let account = &accounts.accounts[0];
        assert!(!account.id.is_empty());
        assert!(!account.name.is_empty());
        assert!(!account.currency.is_empty());
        assert!(account.balance_date > 0);
    }

    #[test]
    fn test_accounts_with_date_range() {
        let bridge = setup_bridge();
        let params = AccountsParams {
            start_date: Some(timestamp(2020, 1, 1)),
            end_date: Some(timestamp(2020, 12, 31)),
            pending: None,
            account_ids: None,
            balances_only: None,
        };

        let accounts = bridge.accounts(Some(params));
        assert!(accounts.errors.is_empty());
    }

    #[test]
    fn test_accounts_with_specific_accounts() {
        let bridge = setup_bridge();

        // First get all accounts to get valid IDs
        let all_accounts = bridge.accounts(None);
        let account_id = all_accounts.accounts[0].id.clone();

        let params = AccountsParams {
            start_date: None,
            end_date: None,
            pending: None,
            account_ids: Some(vec![account_id.clone()]),
            balances_only: None,
        };

        let filtered_accounts = bridge.accounts(Some(params));
        assert!(filtered_accounts.errors.is_empty());
        assert_eq!(filtered_accounts.accounts.len(), 1);
        assert_eq!(filtered_accounts.accounts[0].id, account_id);
    }

    #[test]
    fn test_accounts_balances_only() {
        let bridge = setup_bridge();
        let params = AccountsParams {
            start_date: None,
            end_date: None,
            pending: None,
            account_ids: None,
            balances_only: Some(true),
        };

        let accounts = bridge.accounts(Some(params));
        assert!(accounts.errors.is_empty());
        for account in accounts.accounts {
            assert!(account.transactions.is_none() || account.transactions.unwrap().is_empty());
        }
    }

    #[test]
    fn test_accounts_params_query_string() {
        let params = AccountsParams {
            start_date: Some(1577836800), // 2020-01-01
            end_date: Some(1609459199),   // 2020-12-31
            pending: Some(true),
            account_ids: Some(vec!["123".to_string(), "456".to_string()]),
            balances_only: Some(true),
        };

        let query = params.to_query_string();
        assert!(query.contains("start-date=1577836800"));
        assert!(query.contains("end-date=1609459199"));
        assert!(query.contains("pending=1"));
        assert!(query.contains("account=123"));
        assert!(query.contains("account=456"));
        assert!(query.contains("balances-only=1"));
    }
}
