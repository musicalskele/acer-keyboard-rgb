use crate::{Direction, LightingMode};
use color_eyre::eyre::Result;
use std::str::FromStr;

pub fn parse_lighting_mode(input: &str) -> Result<LightingMode, String> {
    <LightingMode as FromStr>::from_str(input)
        .map_err(|_| "Invalid lighting mode, please try again.".to_string())
}

pub fn parse_direction(input: &str) -> Result<Direction, String> {
    <Direction as FromStr>::from_str(input)
        .map_err(|_| "Invalid direction, please try again.".to_string())
}

pub fn parse_zones(input: &str) -> Result<Vec<u8>, String> {
    input
        .split(',')
        .map(|s| {
            s.trim()
                .parse::<u8>()
                .map_err(|_| "Zones must be numbers separated by commas.".to_string())
        })
        .collect()
}

pub fn parse_u8(input: &str, field: &str, min: u8, max: u8) -> Result<u8, String> {
    let value: u8 = input
        .parse()
        .map_err(|_| format!("{} must be a number.", field))?;
    if value >= min && value <= max {
        Ok(value)
    } else {
        Err(format!("{} must be between {} and {}.", field, min, max))
    }
}

pub fn parse_confirmation(input: &str) -> Result<bool, String> {
    match input.to_lowercase().as_str() {
        "y" | "yes" => Ok(true),
        "n" | "no" => Ok(false),
        _ => Err("Invalid input, please enter 'Y' or 'N'.".to_string()),
    }
}

// function to parse color input in either #rrggbb, #rgb, rrggbb, or r,g,b format
pub fn parse_color(input: &str) -> Result<(u8, u8, u8), String> {
    if let Some(hex) = input.strip_prefix('#') {
        // Handle #rrggbb or #rgb format
        parse_hex_color(hex)
    } else if input.len() == 6 {
        // Handle rrggbb format
        parse_hex_color(input)
    } else if input.contains(',') {
        // Handle r,g,b format
        parse_rgb_tuple(input)
    } else {
        Err("Invalid color format. Use #rrggbb, #rgb, rrggbb, or r,g,b.".to_string())
    }
}

// helper function to parse rrggbb or #rgb/#rrggbb format
pub fn parse_hex_color(hex: &str) -> Result<(u8, u8, u8), String> {
    match hex.len() {
        6 => {
            let red =
                u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid red component in hex")?;
            let green =
                u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid green component in hex")?;
            let blue =
                u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid blue component in hex")?;
            Ok((red, green, blue))
        }
        3 => {
            // expand #rgb to #rrggbb (e.g., #123 -> #112233)
            let red = u8::from_str_radix(&format!("{}{}", &hex[0..1], &hex[0..1]), 16)
                .map_err(|_| "Invalid red component in shorthand hex")?;
            let green = u8::from_str_radix(&format!("{}{}", &hex[1..2], &hex[1..2]), 16)
                .map_err(|_| "Invalid green component in shorthand hex")?;
            let blue = u8::from_str_radix(&format!("{}{}", &hex[2..3], &hex[2..3]), 16)
                .map_err(|_| "Invalid blue component in shorthand hex")?;
            Ok((red, green, blue))
        }
        _ => Err("Hex color must be either 3 or 6 characters long".to_string()),
    }
}

// helper function to parse r,g,b format
pub fn parse_rgb_tuple(input: &str) -> Result<(u8, u8, u8), String> {
    let parts: Vec<&str> = input.split(',').collect();
    if parts.len() != 3 {
        return Err("RGB tuple must have exactly 3 components".to_string());
    }
    let red = parts[0]
        .trim()
        .parse::<u8>()
        .map_err(|_| "Invalid red component in RGB")?;
    let green = parts[1]
        .trim()
        .parse::<u8>()
        .map_err(|_| "Invalid green component in RGB")?;
    let blue = parts[2]
        .trim()
        .parse::<u8>()
        .map_err(|_| "Invalid blue component in RGB")?;
    Ok((red, green, blue))
}

pub fn preview_static_mode(zones: Vec<u8>, red: u8, green: u8, blue: u8) {
    let color_code = format!("\x1b[48;2;{};{};{}m \x1b[0m", red, green, blue); // ANSI code for background color

    println!("\nPreview of static mode (colored blocks):");
    for zone in 1..=4 {
        if zones.contains(&zone) {
            print!("Zone {}: {}\t", zone, color_code);
        } else {
            print!("Zone {}: [-]\t", zone);
        }
    }
    println!("\n");
    for zone in 1..=4 {
        if zones.contains(&zone) {
            print!("{}{} ", color_code, color_code,);
        } else {
            print!("  ");
        }
    }
    println!("\n");
}
