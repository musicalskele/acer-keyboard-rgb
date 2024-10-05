mod interactive;
mod utils;

use utils::{parse_color, preview_static_mode};

use interactive::interactive_mode;

use clap::{Parser, ValueEnum};
use color_eyre::eyre::{eyre, Result, WrapErr};

use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{/* self, */ Write};
use std::path::PathBuf;
use std::str::FromStr;

const PAYLOAD_SIZE: usize = 16;
const PAYLOAD_SIZE_STATIC: usize = 4;
const CHARACTER_DEVICE: &str = "/dev/acer-gkbbl-0";
const CHARACTER_DEVICE_STATIC: &str = "/dev/acer-gkbbl-static-0";
const ALL_ZONES: [Zone; 4] = [Zone(1), Zone(2), Zone(3), Zone(4)];

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum LightingMode {
    Static,
    Breath,
    Neon,
    Wave,
    Shifting,
    Zoom,
}

impl FromStr for LightingMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "static" => Ok(LightingMode::Static),
            "breath" => Ok(LightingMode::Breath),
            "neon" => Ok(LightingMode::Neon),
            "wave" => Ok(LightingMode::Wave),
            "shifting" => Ok(LightingMode::Shifting),
            "zoom" => Ok(LightingMode::Zoom),
            _ => Err(format!("'{}' is not a valid lighting mode", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Direction {
    RightToLeft = 1,
    LeftToRight = 2,
}

impl FromStr for Direction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "right-to-left" => Ok(Direction::RightToLeft),
            "left-to-right" => Ok(Direction::LeftToRight),
            _ => Err(format!("'{}' is not a valid direction", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct RGB {
    red: u8,
    green: u8,
    blue: u8,
}

impl RGB {
    fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    fn to_bytes(self) -> [u8; 3] {
        [self.red, self.green, self.blue]
    }
}

impl std::fmt::Display for RGB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RGB({}, {}, {})", self.red, self.green, self.blue)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Zone(u8);

impl Zone {
    fn new(zone: u8) -> Result<Self> {
        match zone {
            0 => Ok(Self(0)),
            1..=4 => Ok(Self(zone)),
            _ => Err(eyre!("Zone must be 0 (all zones) or between 1 and 4")),
        }
    }

    fn to_mask(self) -> u8 {
        1 << (self.0 - 1)
    }

    fn to_u8(self) -> u8 {
        self.0
    }

    fn zones_to_u8s(zones: Vec<Self>) -> Vec<u8> {
        zones.into_iter().map(|zone| zone.to_u8()).collect()
    }
}

impl std::fmt::Display for Zone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Zone {}", self.0)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Speed(u8);

impl Speed {
    fn new(speed: u8) -> Result<Self> {
        match speed <= 9 {
            true => Ok(Self(speed)),
            false => Err(eyre!("Speed should be between 0 and 9")),
        }
    }
}

impl std::fmt::Display for Speed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Speed {}", self.0)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Brightness(u8);

impl Brightness {
    fn new(brightness: u8) -> Result<Self> {
        match brightness <= 100 {
            true => Ok(Self(brightness)),
            false => Err(eyre!("Brightness must be between 0 and 100")),
        }
    }
}

impl std::fmt::Display for Brightness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Brightness {}%", self.0)
    }
}

#[derive(Parser, Debug, Serialize, Deserialize)]
#[command(about = "Control Predator keyboard RGB lighting")]
struct Args {
    #[arg(
        short,
        long,
        default_value = "static",
        help = "Lighting mode (e.g., wave, static, etc.)"
    )]
    mode: LightingMode,

    #[arg(
        short = 'z',
        long,
        value_delimiter = ',',
        default_value = "0",
        help = "Zones (0 for all, 1-4 for specific zones)"
    )]
    zones: Vec<u8>,

    #[arg(short = 's', long, default_value = "4", help = "Lighting speed (0-9)")]
    speed: u8,

    #[arg(
        short = 'y',
        long,
        default_value = "100",
        help = "Brightness percentage (0-100)"
    )]
    brightness: u8,

    #[arg(
        short = 'd',
        long,
        default_value = "left-to-right",
        help = "Lighting direction (left-to-right or right-to-left)"
    )]
    direction: Direction,

    #[arg(
        long,
        help = "Color in #rrggbb, #rgb, rrggbb, or r,g,b format. overwrites -r,-g,-b."
    )]
    color: Option<String>,

    #[arg(
        short = 'r',
        long,
        default_value = "240",
        help = "Red component of the color (0-255)"
    )]
    red: u8,

    #[arg(
        short = 'g',
        long,
        default_value = "48",
        help = "Green component of the color (0-255)"
    )]
    green: u8,

    #[arg(
        short = 'b',
        long,
        default_value = "32",
        help = "Blue component of the color (0-255)"
    )]
    blue: u8,

    #[arg(long, help = "Save the current profile to a file")]
    save: Option<String>,

    #[arg(long, help = "Load an existing profile from a file")]
    load: Option<String>,

    #[arg(long, help = "List available saved profiles")]
    list: bool,

    #[arg(long, help = "Perform a dry run without applying changes")]
    dry_run: bool,

    #[arg(short, long, help = "Interactive mode to set configurations")]
    interactive: bool,
}

fn convert_zones(zones: &[u8]) -> Result<Vec<Zone>> {
    if zones.len() == 1 && zones[0] == 0 {
        return Ok(ALL_ZONES.to_vec());
    }
    zones.iter().map(|&z| Zone::new(z)).collect()
}

#[derive(Debug)]
struct DevicePayload {
    device: String,
    payload: Vec<u8>,
}

impl std::fmt::Display for DevicePayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Device: {}\nPayload: {:02X?}", self.device, self.payload)
    }
}

enum KeyboardController {
    Real {
        device: Option<File>,
        device_static: Option<File>,
    },
    DryRun,
}

impl KeyboardController {
    fn new(dry_run: bool) -> Result<Self> {
        if dry_run {
            Ok(Self::DryRun)
        } else {
            Ok(Self::Real {
                device: None,
                device_static: None,
            })
        }
    }

    fn open_device(device: &str) -> Result<File> {
        OpenOptions::new()
            .write(true)
            .open(device)
            .wrap_err_with(|| format!("Failed to open device {}", device))
    }

    fn lazy_open_device(&mut self) -> Result<&mut File> {
        if let Self::Real { device, .. } = self {
            if device.is_none() {
                *device = Some(Self::open_device(CHARACTER_DEVICE)?);
            }
            Ok(device.as_mut().unwrap())
        } else {
            Err(eyre!("Dry run mode, no real device"))
        }
    }

    fn lazy_open_static_device(&mut self) -> Result<&mut File> {
        if let Self::Real { device_static, .. } = self {
            if device_static.is_none() {
                *device_static = Some(Self::open_device(CHARACTER_DEVICE_STATIC)?);
            }
            Ok(device_static.as_mut().unwrap())
        } else {
            Err(eyre!("Dry run mode, no real device"))
        }
    }

    fn apply_static(&mut self, zones: &[Zone], color: RGB) -> Result<Vec<DevicePayload>> {
        let mut payloads = Vec::new();
        let mut static_payloads = Vec::new();

        for &zone in zones {
            let mut static_payload = [0u8; PAYLOAD_SIZE_STATIC];
            static_payload[0] = zone.to_mask();
            let [r, g, b] = color.to_bytes();
            static_payload[1..4].copy_from_slice(&[r, g, b]);
            static_payloads.push(static_payload);

            let payload = DevicePayload {
                device: CHARACTER_DEVICE_STATIC.to_string(),
                payload: static_payload.to_vec(),
            };

            payloads.push(payload);
        }

        if let Self::Real { .. } = self {
            let device_static = self.lazy_open_static_device()?;
            for payload in static_payloads {
                device_static
                    .write_all(&payload)
                    .wrap_err("Failed to write static payload")?;
            }
        }

        let mut dynamic_payload = [0u8; PAYLOAD_SIZE];
        dynamic_payload[2] = 100; // brightness
        dynamic_payload[9] = 1;

        let payload = DevicePayload {
            device: CHARACTER_DEVICE.to_string(),
            payload: dynamic_payload.to_vec(),
        };

        if let Self::Real { .. } = self {
            let device = self.lazy_open_device()?;
            device
                .write_all(&dynamic_payload)
                .wrap_err("Failed to write dynamic payload")?;
        }

        payloads.push(payload);
        Ok(payloads)
    }

    fn apply_dynamic(
        &mut self,
        mode: LightingMode,
        speed: Speed,
        brightness: Brightness,
        direction: Direction,
        color: RGB,
    ) -> Result<Vec<DevicePayload>> {
        let mut payload = [0u8; PAYLOAD_SIZE];
        payload[0] = mode as u8;
        payload[1] = speed.0;
        payload[2] = brightness.0;
        payload[3] = if matches!(mode, LightingMode::Wave) {
            8
        } else {
            0
        };
        payload[4] = direction as u8;
        let [r, g, b] = color.to_bytes();
        payload[5..8].copy_from_slice(&[r, g, b]);
        payload[9] = 1;

        let device_payload = DevicePayload {
            device: CHARACTER_DEVICE.to_string(),
            payload: payload.to_vec(),
        };

        if let Self::Real { .. } = self {
            let device = self.lazy_open_device()?;
            device
                .write_all(&payload)
                .wrap_err("Failed to write dynamic payload")?;
        }

        Ok(vec![device_payload])
    }
}

fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_default()
        .join("predator/profiles")
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut args = Args::parse();

    if args.interactive {args = interactive_mode();}

    let (mut red, mut green, mut blue) = (args.red, args.green, args.blue);

    if let Some(color_input) = &args.color {
        let parsed_color = parse_color(color_input)
            .map_err(|e| eyre!(e))
            .wrap_err("Failed to parse color input")?;
        red = parsed_color.0;
        green = parsed_color.1;
        blue = parsed_color.2;
    }

    args.red = red;
    args.green = green;
    args.blue = blue;

    let config_dir = get_config_dir();
    std::fs::create_dir_all(&config_dir).wrap_err("Failed to create config directory")?;

    if args.list {
        println!("Saved profiles:");
        for entry in std::fs::read_dir(&config_dir)? {
            let entry = entry?;
            if let Some(name) = entry.path().file_stem() {
                println!("\t{}", name.to_string_lossy());
            }
        }
        return Ok(());
    }

    let args = if let Some(profile) = args.load.as_ref() {
        let path = config_dir.join(format!("{}.json", profile));
        serde_json::from_reader(
            File::open(path).wrap_err_with(|| format!("Failed to load profile '{}'", profile))?,
        )
        .wrap_err("Failed to parse profile")?
    } else {
        args
    };

    if let Some(profile) = args.save.as_ref() {
        let path = config_dir.join(format!("{}.json", profile));
        serde_json::to_writer_pretty(
            File::create(&path)
                .wrap_err_with(|| format!("Failed to create profile file '{}'", profile))?,
            &args,
        )
        .wrap_err("Failed to save profile")?;
    }

    let mut controller = KeyboardController::new(args.dry_run)?;
    let color = RGB::new(args.red, args.green, args.blue);
    let speed = Speed::new(args.speed)?;
    let brightness = Brightness::new(args.brightness)?;

    let zones = convert_zones(&args.zones)?;

    println!("Configuration:");
    println!("Mode: {:?}", args.mode);
    println!("Zones: {:?}", zones);
    println!("Color: {}", color);
    println!("{}", speed);
    println!("{}", brightness);
    println!("Direction: {:?}", args.direction);

    let payloads = match args.mode {
        LightingMode::Static => controller.apply_static(&zones, color)?,
        _ => controller.apply_dynamic(args.mode, speed, brightness, args.direction, color)?,
    };

    let zones_u8 = Zone::zones_to_u8s(zones);

    preview_static_mode(zones_u8, red, green, blue);
    if args.dry_run {
        println!("\nDevice Payloads:");
        for payload in payloads {
            println!("{}\n", payload);
        }
    }

    Ok(())
}
