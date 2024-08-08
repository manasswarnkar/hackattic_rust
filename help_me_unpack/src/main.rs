use std::{ collections::HashMap, env, };
use dotenv::dotenv;
use reqwest::Client;
use serde::Serialize;
use base64::prelude::*;

extern crate tokio;


async fn get_data(access_token: &str) -> Result<String, Box<dyn std::error::Error>> {
    let str = format!("https://hackattic.com/challenges/help_me_unpack/problem?access_token={}", access_token);

    let mut resp = reqwest::get(&str)
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    let bytes = resp.remove("bytes").unwrap();
    
    Ok(bytes)
}


async fn post_data(access_token: &str, ans : Pack) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!("https://hackattic.com/challenges/help_me_unpack/solve?access_token={}", access_token);
    let response = client.post(url).json(&ans).send().await?;
    println!("Body: {}", response.text().await?);

    Ok(())
}

#[derive(Serialize, Default, Debug)]
struct Pack {
    int : i32,
    uint : u32,
    short : i16,
    float : f32,
    double : f64,
    big_endian_double : f64
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    dotenv().ok();
    let access_token = env::var("ACCESS_TOKEN").expect("ACCESS_TOKEN must be set in env file");
    let bytes = get_data(&access_token).await?;
    let decoded_bytes = BASE64_STANDARD.decode(bytes).unwrap();

    let mut arry : [u8; 4] = decoded_bytes[0..4].try_into().expect("slice len should be 4");
    let a = Some(i32::from_le_bytes(arry)).unwrap();

    arry = decoded_bytes[4..8].try_into().expect("2");
    let b = Some(u32::from_le_bytes(arry)).unwrap();

    let arry : [u8; 2] = decoded_bytes[8..10].try_into().expect("3");
    let c = Some(i16::from_le_bytes(arry)).unwrap();
    
    //10..12 not used

    let arry : [u8; 4] = decoded_bytes[12..16].try_into().expect("4");
    let d = Some(f32::from_le_bytes(arry)).unwrap();

    let mut arry : [u8; 8] = decoded_bytes[16..24].try_into().expect("5");
    let e = Some(f64::from_le_bytes(arry)).unwrap();

    arry = decoded_bytes[24..32].try_into().expect("6");
    let f = Some(f64::from_be_bytes(arry)).unwrap();

    let ans = Pack {int : a, uint : b, short : c, float : d, double : e, big_endian_double : f};

    post_data(&access_token, ans).await?;

    Ok(())
}
