use std::collections::HashMap;
use base64::prelude::*;
use serde::Serialize;
use serde_json::{json, Value};


#[derive(Serialize, Default, Debug)]
struct Pack {
    int : i32,
    uint : u32,
    short : i16,
    float : f32,
    double : f64,
    big_endian_double : f64
}

pub fn solve(mut raw_data: HashMap<String, String>) -> Value {
    let bytes = raw_data.remove("bytes").unwrap();
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

    let resp = json!(ans);

    resp
}