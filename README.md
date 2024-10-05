a simple command-line tool for managing the keyboard RGB lighting on Acer laptops.

## features

- control RGB lighting modes (e.g., static, wave, etc.)
- adjust brightness, speed, and color
- set lighting zones
- save and load lighting profiles

## usage

```bash
acer-rgb [OPTIONS]
```

for detailed options, run:

```bash
acer-rgb --help
```

```
Usage: acer-rgb [OPTIONS]

Options:
  -m, --mode <MODE>              Lighting mode (e.g., wave, static, etc.) [default: static] [possible values: static, breath, neon, wave, shifting, zoom]
  -z, --zones <ZONES>            Zones (0 for all, 1-4 for specific zones) [default: 0]
  -s, --speed <SPEED>            Lighting speed (0-9) [default: 4]
  -y, --brightness <BRIGHTNESS>  Brightness percentage (0-100) [default: 100]
  -d, --direction <DIRECTION>    Lighting direction (left-to-right or right-to-left) [default: left-to-right] [possible values: right-to-left, left-to-right]
      --color <COLOR>            Color in #rrggbb, #rgb, rrggbb, or r,g,b format. overwrites -r,-g,-b.
  -r, --red <RED>                Red component of the color (0-255) [default: 240]
  -g, --green <GREEN>            Green component of the color (0-255) [default: 48]
  -b, --blue <BLUE>              Blue component of the color (0-255) [default: 32]
      --save <SAVE>              Save the current profile to a file
      --load <LOAD>              Load an existing profile from a file
      --list                     List available saved profiles
      --dry-run                  Perform a dry run without applying changes
  -i, --interactive              Interactive mode to set configurations
  -h, --help                     Print help
```

### example:
```
acer-rgb -m static -z 0 -color #ff0000
```
sets all zones to pure red
## building

clone the repository and build the project:

```bash
git clone https://github.com/musicalskele/acer-keyboard-rgb.git
cd acer-rgb
cargo build --release
```
--
## license

this project is licensed under the **GNU Affero General Public License Version 3**. see the **LICENSE** file for more details.  