use std::{collections::HashMap, env, io};
use dotenv::dotenv;
use reqwest::Client;
use serde_json::Value;

mod challenges;

async fn get(
    challenge: &str,
    access_token: &str,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let url = format!(
        "https://hackattic.com/challenges/{}/problem?access_token={}",
        challenge, access_token
    );

    let resp = reqwest::get(&url)
        .await?
        .json::<HashMap<String, String>>()
        .await?;

    Ok(resp)
}

async fn post(
    challenge: &str,
    access_token: &str,
    ans: Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!(
        "https://hackattic.com/challenges/{}/solve?access_token={}&playground=1",
        challenge, access_token
    );
    let response = client.post(url).json(&ans).send().await?;
    println!("Body: {}", response.text().await?); //

    Ok(())
}

type SolverFn = fn(HashMap<String, String>) -> Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let access_token: String = env::var("ACCESS_TOKEN").expect("ACCESS_TOKEN must be set in env file.");
    
    let mut challenge = String::new();
    io::stdin()
        .read_line(&mut challenge)
        .expect("failed to read line");

    let challenge = challenge.trim();

    println!("{}", &challenge);
    let function_map: HashMap<String, SolverFn> = HashMap::from([
        ("help_me_unpack".to_string(), challenges::help_me_unpack::solve as SolverFn),
        ("reading_qr".to_string(), challenges::reading_qr::solve as SolverFn),
    ]);

    if let Some(func) = function_map.get(challenge) {
        println!("challenge found!");
        let raw_data = get(&challenge, &access_token).await?;
        let ans = func(raw_data);
        post(&challenge, &access_token, ans).await?;
    } else {
        println!("challenge not found");
    }

    Ok(())
}
