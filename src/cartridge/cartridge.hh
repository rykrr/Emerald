#pragma once
#include <memory>
#include <string>
#include <iomanip>
#include <iostream>
#include <exception>
#include <fstream>

#include "../memory.hh"
#include "../macros.hh"

#include "../core/registers.hh"


#define CARTRIDGE_TYPE_ADDR 0x0174


struct rom_s {
	std::unique_ptr<char[]> data;
	u32 length;
};


class Cartridge {

	rom_s boot_rom;	// 0000 -> 00FF
	rom_s cart_rom;	// Cartridge contents
	
	u8 *banks[2];	// 0000 -> 3FFF, 4000 -> 7FFF
	u8 *ram;		// A000 -> BFFF
	
private:
	
	rom_s read(std::string path) {
		
		rom_s rom;
		std::ifstream file(path, std::ios::binary | std::ios::ate);

		if (!file.good())
			throw std::runtime_error("Could not read " + path);

		rom.length = static_cast<u32>(file.tellg());
		rom.data = std::unique_ptr<char[]>(new char[rom.length]);

		file.seekg(0, std::ios::beg);
		file.read(rom.data.get(), rom.length);
		
		file.close();
		return rom;
	}
	
	
	inline u8 &io_callback(u16 addr) {
		static u8 dummy;
		return dummy;
	}
	
public:

	void load_boot_rom(std::string path) {

		boot_rom = read(path);

		memory.copy(0x00, reinterpret_cast<u8*>(boot_rom.data.get()),
			boot_rom.length < 0x100? boot_rom.length : 0x100);
	}

	void load_rom(std::string path, bool override_boot=false) {

		cart_rom = read(path);

		if (override_boot || cart_rom.length < 0x100)
			memory.copy(0x00, reinterpret_cast<u8*>(cart_rom.data.get()),
				cart_rom.length < 0xBF00? cart_rom.length : 0xBF00);
		else
			memory.copy(0x100, reinterpret_cast<u8*>(cart_rom.data.get() + 0x100),
				cart_rom.length < 0xBE00? cart_rom.length - 0x100 : 0xBE00);
	}
	
	void copy_logo() {
		
		memory.copy(0x104, reinterpret_cast<u8*>(boot_rom.data.get()) + 0xA8, 0x31);
	}

	Cartridge() {

		cart_rom.length = 0;
		boot_rom.length = 0;

		memory.init_register(0x50, [this](u16 addr, u8 value, bool write) {

			if (write && value)
				memory.copy(0x00, reinterpret_cast<u8*>(cart_rom.data.get()),
					cart_rom.length < 0x100? cart_rom.length : 0x100);
			
			return value;
		});
		
		memory.set_bank_controller([](u16 a, u8 b, bool w) {
#ifdef VDEBUG
			std::cout	<< "rom access at " << to_hex(a, 4)
						<< " value " << to_hex(b, 2)
						<< " pc " << to_hex(Registers::PC, 4)
						<< std::endl;
#endif
			return 0xFF;
		});
	};
};

extern Cartridge cartridge;
