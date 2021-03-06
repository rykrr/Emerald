#include <csignal>
#include <unistd.h>

#include "graphics/graphics.hh"

#include "interrupt.hh"
#include "memory.hh"
#include "cartridge.hh"
#include "clock.hh"

#include "core/registers.hh"
#include "core/timer.hh"
#include "core/cpu.hh"
#include "core/mappings.hh"

RegisterFile registers;

#define BIND_BYTE(R) ByteRegister Registers::R = registers.R;
#define BIND_WORD(R) WordRegister Registers::R = registers.R;

BIND_WORD(AF); BIND_BYTE(A); BIND_BYTE(F);
BIND_WORD(BC); BIND_BYTE(B); BIND_BYTE(C);
BIND_WORD(DE); BIND_BYTE(D); BIND_BYTE(E);
BIND_WORD(HL); BIND_BYTE(H); BIND_BYTE(L);
BIND_WORD(SP);
BIND_WORD(PC);

#undef BIND_BYTE
#undef BIND_WORD

#define GUARD_REGISTER_VALUE 0xABCD
static u16 guard_register = GUARD_REGISTER_VALUE;

ByteRegister Registers::_ = *((u8 *) &guard_register);
WordRegister Registers::__ = guard_register;

Memory memory;
Cartridge cartridge;
	
Graphics graphics;

Timer timer;
Clock clk;
CPU cpu;


#ifdef DEBUG
#include "debug/ndebug.hh"
#endif

void stop(int s) {
	
	//exit(0);
	throw std::runtime_error("Execution Terminated by User");
	signal(SIGINT, stop);
}

void init() {

	using namespace Interrupt;
	using namespace Registers;

	signal(SIGINT, stop);

	AF = 0x01B0;
	BC = 0x0013;
	DE = 0x00DE;
	HL = 0x014D;
	PC = 0x0000;
	SP = 0xFFFE;

	// Clear IO registers
	for (u8 i = 0; i < 0xFF; i++)
		memory[0xFF00 + i].write_byte(0);

	memory[0xFF40].write_byte(0x91); // LCDC
	memory[0xFF41].write_byte(0x01); // STAT
	memory[0xFF47].write_byte(0xFC); // BGP
	memory[0xFF48].write_byte(0xFF); // OBP0
	memory[0xFF48].write_byte(0xFF); // OBP0
	
	clk.add(timer);
	clk.add(graphics);

#ifdef DEBUG
	clk.add(debugger);
	debugger += 0;
#endif
	
	timer.disable();
	cpu.disable_interrupts(TIMER);
	cpu.reset();
}

int main(int argc, char **argv) {
	
	using namespace Registers;
	using namespace std;

	int opt;
	bool override_cartridge_logo= false;
	string boot_rom = "boot.gb";
	string cart_rom = "cart.gb";

	while ((opt = getopt(argc, argv, "b:c:l")) != -1) {
		switch(opt) {
			case 'b':
				boot_rom = string(optarg);
				break;

			case 'c':
				cart_rom = string(optarg);
				break;

			case 'l':
				override_cartridge_logo = true;
				break;

			default:
				break;
		}
	}

	init();

#ifdef VDEBUG
	cout << "Boot: " << boot_rom << endl;
	cout << "Cart: " << cart_rom << endl;
#endif
	cartridge.load_boot_rom(boot_rom);
	cartridge.load_rom(cart_rom);

#ifdef VDEBUG
	cout << "\n-- Logo ROM --" << endl;
	for (u8 i = 0; i < 0x31; i++)
		cout	<< to_hex(memory[0xA8 + i].read_byte(), 2) << "  "
				<< to_hex(memory[0x104 + i].read_byte(), 2) << endl;
	cout << "-- Logo ROM --\n" << endl;
#endif

	if (override_cartridge_logo)
		cartridge.copy_logo();
	
	try {
		cpu.run();
	}
	catch (std::exception &e) {
#ifdef DEBUG
		debugger.fatal();
#endif
	}
}
