mod config;

use std::{fs::{self, File}, io::BufReader, process, time::Duration};
use battery::{Battery, Manager};
use config::Config;
use reqwest::Client;
use tokio::time;

#[tokio::main]
async fn main() {
    if fs::metadata("config.json").is_err() {
        println!("config.json could not be found!");
        process::exit(1);
    }

    println!("Reading values from config");
    let config = get_config();
    println!("Config read");

    let off_webhook_id = config.charger_off_webhook_id;
    let on_webhook_id = config.charger_on_webhook_id;
    let token = config.token;
    let url = config.home_assistant_url;

    let manager = Manager::new().expect("Unable to create battery manager");
    
    let batteries: Vec<_> = manager.batteries().expect("Unable to get batteries")
        .map(|b| b.expect("Unable to access battery"))
        .collect();

    if batteries.is_empty() {
        println!("Your device does not have a battery!");
        process::exit(0);
    }

    let battery = batteries.first().unwrap();
    let percentage = get_soc(battery);

    let mut is_charging = false;

    loop {
        if percentage <= 20.0 && !is_charging {
            println!("Turning on charger.");
            turn_on_charger(url.clone(), on_webhook_id.clone(), token.clone()).await;
            
            is_charging = true;
        } else if percentage == 100.0 && is_charging {
            println!("Turning off charger.");
            turn_off_charger(url.clone(), off_webhook_id.clone(), token.clone()).await;
            
            is_charging = false;
        }

        let _ = time::sleep(Duration::from_secs(30));
    }

}

async fn turn_off_charger(home_assistant_url: String, charger_off_webhook_id: String, token: String) {
    _ = Client::new()
        .post(format!("{}/api/webhook/{}", home_assistant_url, charger_off_webhook_id))
        .bearer_auth(token)
        .send()
        .await;

    println!("Charger turned off");
}

async fn turn_on_charger(home_assistant_url: String, charger_on_webhook_id: String, token: String) {
    _ = Client::new()
    .post(format!("{}/api/webhook/{}", home_assistant_url, charger_on_webhook_id))
    .bearer_auth(token)
    .send()
    .await;

    println!("Charger turned on");
}

fn get_soc(battery: &Battery) -> f32 {
    battery.state_of_charge().get::<battery::units::ratio::percent>()
}

fn get_config() -> Config {
    let file = File::open("config.json").unwrap();
    let reader = BufReader::new(file);

    serde_json::from_reader(reader).expect("Error parsing JSON")
}