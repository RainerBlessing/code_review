use serde::Deserialize;
#[derive(Deserialize, Debug)]
pub struct Config {
    pub model: String,
    pub url: String,
}
