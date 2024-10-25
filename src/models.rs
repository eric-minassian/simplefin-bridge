use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
pub struct Info {
    pub versions: Vec<String>,
}
