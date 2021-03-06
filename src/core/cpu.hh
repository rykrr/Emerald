#pragma once
#include <vector>
#include <functional>

#include "registers.hh"

#include "../memory.hh"
#include "../clock.hh"
#include "../macros.hh"


class CPU {
	
	bool IME;
	u8 IF, IE;

	bool on;
	bool halted;
	u8 instruction_set;
	
private:
	
	void handle_interrupts();
	
public:
	
	void enable_interrupts(u8 mask, bool enable_ime = false);
	void disable_interrupts(u8 mask, bool disable_ime = false);
	
	void stop() { on = false; puts("Stopped."); }
	void halt() { halted = true; puts("Halted."); }
	
	void reset() { on = true; halted = false; }
	
	void cb() { instruction_set = 1; };
	
	void run();
	
	void push(WordRegister);
	void pop(WordRegister);
	
public:
	
	CPU()
		:	IME(true), IF(0), IE(0),
			instruction_set(0),
			halted(false),
			on(true) {
		
		memory.init_register(0x0F, IF);
		memory.init_register(0x80, IE);
	}
};


extern CPU cpu;
