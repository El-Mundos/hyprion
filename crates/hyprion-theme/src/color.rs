use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let hex = hex.trim_start_matches('#');

        let (r, g, b, a) = match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|e| format!("Invalid red channel: {}", e))?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|e| format!("Invalid green channel: {}", e))?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|e| format!("Invalid blue channel: {}", e))?;
                (r, g, b, 255)
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|e| format!("Invalid red channel: {}", e))?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|e| format!("Invalid green channel: {}", e))?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|e| format!("Invalid blue channel: {}", e))?;
                let a = u8::from_str_radix(&hex[6..8], 16)
                    .map_err(|e| format!("Invalid alpha channel: {}", e))?;
                (r, g, b, a)
            }
            _ => return Err(format!("Invalid hex color length: #{}", hex)),
        };

        Ok(Color { r, g, b, a })
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Color::from_hex(&s).map_err(serde::de::Error::custom)
    }
}
