use crate::utils::*;

use crate::{Args, Direction, LightingMode};
use color_eyre::eyre::Result;
use dialoguer::Input;

pub fn interactive_mode() -> Args {
    let mut args;

    loop {
        args = gather_args(None);

        println!("\nHere are the selected arguments:");
        println!("Mode: {:?}", args.mode);
        println!("Zones: {:?}", args.zones);
        println!("Speed: {}", args.speed);
        println!("Brightness: {}", args.brightness);
        println!("Direction: {:?}", args.direction);
        println!("Color: RGB({}, {}, {})", args.red, args.green, args.blue);

        let confirmation =
            prompt_with_retry("Apply these settings? (Y/n)", "Y", parse_confirmation);

        if confirmation {
            break;
        } else {
            println!("Let's try again!");
        }
    }

    args
}

fn gather_args(prev_args: Option<&Args>) -> Args {
    let default_mode_str =
        prev_args.map_or("static".to_string(), |args| format!("{:?}", args.mode));
    let mode = prompt_with_retry(
        "Choose lighting mode (static, wave, etc.)",
        &default_mode_str,
        parse_lighting_mode,
    );

    let default_zones_str = prev_args.map_or("0".to_string(), |args| format!("{:?}", args.zones));
    let mut zones = prompt_with_retry(
        "Specify zones (0 for all, or comma-separated for specific zones)",
        &default_zones_str,
        parse_zones,
    );

    if zones.contains(&0) {
        zones = vec![1, 2, 3, 4];
    }

    let speed = if mode != LightingMode::Static {
        let default_speed_str = prev_args.map_or("4".to_string(), |args| args.speed.to_string());
        Some(prompt_with_retry(
            "Lighting speed (0-9)",
            &default_speed_str,
            |input| parse_u8(input, "Speed", 0, 9),
        ))
    } else {
        None
    };

    let brightness = if mode != LightingMode::Static {
        let default_brightness_str =
            prev_args.map_or("100".to_string(), |args| args.brightness.to_string());
        Some(prompt_with_retry(
            "Brightness (0-100)",
            &default_brightness_str,
            |input| parse_u8(input, "Brightness", 0, 100),
        ))
    } else {
        None
    };

    let direction = if mode != LightingMode::Static {
        let default_direction_str = prev_args.map_or("left-to-right".to_string(), |args| {
            format!("{:?}", args.direction)
        });
        Some(prompt_with_retry(
            "Direction (left-to-right or right-to-left)",
            &default_direction_str,
            parse_direction,
        ))
    } else {
        None
    };

    let default_color_str = prev_args.map_or("50,255,50".to_string(), |args| {
        format!("{},{},{}", args.red, args.green, args.blue)
    });
    let (red, green, blue) = prompt_with_retry(
        "Specify color (#rrggbb, #rgb, rrggbb, or r,g,b)",
        &default_color_str,
        parse_color,
    );

    let dry_run = prompt_with_retry("Debug mode? (y/N)", "N", parse_confirmation);

    if mode == LightingMode::Static {
        preview_static_mode(zones.clone(), red, green, blue);
    }

    Args {
        mode,
        zones,
        speed: speed.unwrap_or(4),
        brightness: brightness.unwrap_or(100),
        direction: direction.unwrap_or(Direction::LeftToRight),
        red,
        green,
        blue,
        color: None,
        save: None,
        load: None,
        list: false,
        dry_run,
        interactive: false,
    }
}

fn prompt_with_retry<T, F>(prompt_message: &str, default_value: &str, parse_fn: F) -> T
where
    F: Fn(&str) -> Result<T, String>,
{
    loop {
        let input: String = Input::new()
            .with_prompt(prompt_message)
            .default(default_value.to_string())
            .interact_text()
            .unwrap();

        match parse_fn(&input) {
            Ok(value) => break value,
            Err(err) => eprintln!("{}", err),
        }
    }
}
