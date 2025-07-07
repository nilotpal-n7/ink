use flate2::write::{ZlibEncoder, ZlibDecoder};
use flate2::Compression;
use std::fs::{read_to_string, write};
use std::io::Write;
use std::path::Path;
use anyhow::{Result, anyhow};

/// Compresses the input using zlib (if zip is enabled), or stores raw
pub fn compress(content: Vec<u8>) -> Result<Vec<u8>> {
    let zip = load_is_zip()?; // Default to true if config unreadable

    if zip {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&content)?;
        Ok(encoder.finish()?)
    } else {
        Ok(content)
    }
}

/// Decompresses zlib-compressed blob or returns raw if zip is disabled
pub fn decompress(data: Vec<u8>) -> Result<Vec<u8>> {
    let zip = load_is_zip()?; // Default to true if config unreadable

    if zip {
        let mut decoder = ZlibDecoder::new(Vec::new());
        decoder.write_all(&data)?;
        Ok(decoder.finish()?)
    } else {
        Ok(data)
    }
}

pub fn load_is_zip() -> Result<bool> {
    let path = Path::new(".ink/config");
    let contents = read_to_string(path)?;

    for line in contents.lines() {
        if let Some((key, value)) = line.split_once('=') {
            if key.trim() == "zip" {
                return Ok(string_to_bool(value.trim())?);
            }
        }
    }
    Ok(false)
}

pub fn save_is_zip(is_zip: bool) -> Result<()> {
    let path = Path::new(".ink/config");

    let mut lines = if path.exists() {
        read_to_string(path)?
            .lines()
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let mut found = false;
    for line in lines.iter_mut() {
        if line.trim_start().starts_with("zip=") {
            *line = format!("zip={}", is_zip.to_string());
            found = true;
            break;
        }
    }

    if !found {
        lines.push(format!("zip={}", is_zip.to_string()));
    }

    let output = lines.join("\n") + "\n";
    write(path, output)?; // .as_bytes()

    Ok(())
}

pub fn string_to_bool(input: &str) -> Result<bool> {
    input.parse::<bool>().map_err(|e| anyhow!("Invalid boolean: {}", e))
}
