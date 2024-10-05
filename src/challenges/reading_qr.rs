use std::{collections::HashMap, io::Cursor};
use serde::Serialize;
use serde_json::{json, Value};
use image::ImageReader;
use opencv::{imgcodecs, imgproc, objdetect::QRCodeDetector, prelude::*};
use reqwest;

#[derive(Serialize)]
struct QrCode {
    code: String,
}

fn download_image(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let response = client.get(url).send()?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to download image: {}", response.status()).into());
    }
    Ok(response.bytes()?.to_vec())
}

fn process_image_bytes(bytes: &[u8]) -> Result<Mat, Box<dyn std::error::Error>> {
    let img = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()?
        .decode()?;

    let mut buffer = Vec::new();
    img.write_to(&mut Cursor::new(&mut buffer), image::ImageFormat::Png)?;
    
    let mat = imgcodecs::imdecode(&opencv::core::Mat::from_slice(&buffer)?, imgcodecs::IMREAD_COLOR)?;
    Ok(mat)
}

fn detect_qr_code(img: &Mat) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut gray = Mat::default();
    imgproc::cvt_color(&img, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;
    let detector = QRCodeDetector::default()?;
    let mut points = Mat::default();
    
    if !detector.detect(&gray, &mut points)? {
        return Ok(None);
    }
    let mut straight_qrcode = Mat::default();
    let decoded_bytes = detector.decode(&gray, &points, &mut straight_qrcode)?;
    if decoded_bytes.is_empty() {
        return Ok(None);
    }

    match String::from_utf8(decoded_bytes) {
        Ok(text) => Ok(Some(text)),
        Err(_) => Ok(None)
    }
}

fn process_image(url: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let bytes = download_image(url)?;

    let img = process_image_bytes(&bytes)?;
    
    match detect_qr_code(&img)? {
        Some(decoded_text) => {
            println!("Successfully decoded QR code: {}", decoded_text);
            Ok(json!(QrCode { code: decoded_text }))
        }
        None => {
            eprintln!("No QR code detected");
            Ok(Value::Null)
        }
    }
}

pub fn solve(mut raw_data: HashMap<String, String>) -> Value {
    let url = match raw_data.remove("image_url") {
        Some(url) => url,
        None => return Value::Null,
    };

    match tokio::task::block_in_place(|| process_image(&url)) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error processing image: {}", e);
            Value::Null
        }
    }
}