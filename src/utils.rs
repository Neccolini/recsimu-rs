use once_cell::sync::Lazy;
use serde::Deserialize;
use std::error;
use std::path::PathBuf;
use std::sync::Mutex;

pub static DEBUG_ENABLED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if *crate::utils::DEBUG_ENABLED.lock().unwrap() {
            println!($($arg)*);
        }
    }
}

// pathで指定されたjsonファイルを読み込む
pub fn read_json<T>(path: PathBuf) -> Result<T, Box<dyn error::Error>>
where
    for<'de> T: Deserialize<'de>,
{
    let json_str = std::fs::read_to_string(path)?;
    let json: T = serde_json::from_str(&json_str)?;
    Ok(json)
}

// pathで指定されたjsonファイルに書き込む
pub fn write_json<T>(path: PathBuf, json: &T) -> Result<(), Box<dyn error::Error>>
where
    T: serde::Serialize,
{
    let json_str = serde_json::to_string_pretty(json)?;
    std::fs::write(path, json_str)?;
    Ok(())
}

use num_traits::One;

pub fn div_ceil<T>(dividend: T, divisor: T) -> T
where
    T: std::ops::Div<Output = T>
        + std::ops::Add<Output = T>
        + std::cmp::PartialOrd
        + std::ops::Sub<Output = T>
        + std::ops::Mul<Output = T>
        + Copy
        + One,
{
    (dividend + divisor - T::one()) / divisor
}
