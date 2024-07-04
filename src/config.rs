use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub home_assistant_url: String,
    pub charger_off_webhook_id: String,
    pub charger_on_webhook_id: String,
    pub token: String
}