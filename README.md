# Emerald Gameboy Emulator

![Tetris Title Screen](docs/tetris.png)

## Description
Yet another attempt at a Gameboy emulator (this time in C++).

## Running Emerald
Run ```make``` to compile the program.

Emerald requires a separate ```./emerald -b <path_to_boot.rom> -c <path_to_game.rom>```
It is recommended to run Emerald with the default DMG bootloader.

### Run options
| Options            | Description                                       |
| ------------------ | ------------------------------------------------- |
| -c rom_file        | The game rom to run                               |
| -C rom_file        | The game rom to run without a boot loader         |
| -b rom_file        | The boot rom to run before the game               |

## Debug Console
To enable the debug console, build with the ```debug``` target.
To specify a bootloader other than ```boot.rom```, use the ```-b <path to boot rom>``` flag.

## Checklist
- Implement memory bank controllers for other cartridges
- Implement button inputs
- Implement sound
- Separate graphics into dynamic module
- Rewrite in Rust? Maybe not.
	- This implementation is built around bad reference practices...
	- Look no further than memory.hh
- Simplify clock interface
- Add useful error messages (as opposed to none)

## Known Issues
- Barely runs Tetris (gets stuck on demo screen)
- Large number of failed Blaarg tests
- Will not run Kirby's Dreamland

![It's stuck on this demo screen](docs/tetris2.png)
