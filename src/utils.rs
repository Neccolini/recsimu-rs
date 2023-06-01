use serde::Deserialize;
use std::error;
use std::path::PathBuf;

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
