extern crate image;
extern crate opencv;

use dotenv::dotenv;
use image::ImageReader;
use opencv::{imgcodecs, imgproc, objdetect::QRCodeDetector, prelude::*};
use serde::Serialize;
use std::{collections::HashMap, env, error, io::Cursor};

#[derive(Serialize)]
struct QrCode {
    code: String,
}

async fn get_img(access_token: &str) -> Result<Mat, Box<dyn error::Error>> {
    let str = format!(
        "https://hackattic.com/challenges/reading_qr/problem?access_token={}",
        access_token
    );
    let mut resp = reqwest::get(&str)
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    let url = resp.remove("image_url").unwrap();
    let img = reqwest::get(url).await?;
    if !img.status().is_success() {
        return Err(format!("failed to download image: {}", img.status()).into());
    }
    let bytes = img.bytes().await?;

    let img2 = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()?
        .decode()?;
    let mut buffer = Vec::new();
    img2.write_to(&mut Cursor::new(&mut buffer), image::ImageFormat::Png)?;
    
    let mat = imgcodecs::imdecode(&opencv::core::Mat::from_slice(&buffer)?, imgcodecs::IMREAD_COLOR)?;
    println!("get_img success");
    Ok(mat)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let access_token: String =
        env::var("ACCESS_TOKEN").expect("ACCESS_TOKEN must be set in env file.");

    let img = get_img(&access_token).await?;
    let mut gray = Mat::default();
    imgproc::cvt_color(&img, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;

    let detector = QRCodeDetector::default()?;

    let mut points = Mat::default();
    let found = detector.detect(&gray, &mut points)?;
    if found {
        println!("QR Code detected!");
        let mut straight_qrcode = Mat::default();
        let decoded_bytes = detector.decode(&gray, &points, &mut straight_qrcode)?;

        if !decoded_bytes.is_empty() {
            println!("Decoded bytes: {:?}", decoded_bytes);
            let decoded_text = match String::from_utf8(decoded_bytes) {
                Ok(text) => text,
                Err(_) => {
                    eprintln!("Decoded data is not valid UTF-8");
                    return Err("Decoded data is not valid UTF-8".into());
                }
            };

            println!("Decoded text: {:?}", decoded_text);

            let qr_code = QrCode { code: decoded_text };
            // let json_string = serde_json::to_string(&qr_code)?;
            // println!("json_string: {}", json_string);

            let client = reqwest::Client::new();
            let post_url = format!(
                "https://hackattic.com/challenges/reading_qr/solve?access_token={}",
                access_token
            );
            let post_response = client
                .post(post_url.as_str())
                .json(&qr_code)
                .send()
                .await?;

            if post_response.status().is_success() {
                println!("Success: {}", post_response.text().await?);
            } else {
                println!("Failed to post JSON data: {:?}", post_response.status());
            }
        } else {
            eprintln!("Failed to decode QR code.");
        }
    } else {
        eprintln!("No QR code detected.");
    }

    Ok(())
}
