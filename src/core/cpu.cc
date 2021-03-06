#include "cpu.hh"
#include "mappings.hh"
#include "registers.hh"
#include "../interrupt.hh"
#include "../clock.hh"

using namespace Registers;


void CPU::handle_interrupts() {
	
	// No interrupts to handle
	if (!IF) return;
	
	// Interrupts disabled in normal operation
	if (!halted && !IME) return;

	// Wake up processor
	halted = false;
	
	// Lower bit has highest priority
	for (u8 i = 0; i < 5; i++) {
		
		if (IF & IE & (1 << i)) {
			
			// CALL interrupt vector
			push(PC);
			PC = 0x40 + 8 * i;
			
			// Clear interrupt
			IF ^= (1 << i);
			
			// Disable all interrupts
			IME = false;
		}
	}
}


void CPU::enable_interrupts(u8 mask, bool enable_ime) {
	
	IME |= enable_ime;
	IE |= mask;
}


void CPU::disable_interrupts(u8 mask, bool disable_ime) {
	
	IME &= !disable_ime;
	IE &= ~mask;
}


void CPU::push(WordRegister r) {
	
	SP -= 2;

	if (&r == &AF)
		memory[SP].write_word(r & 0xFFF0);
	else
		memory[SP].write_word(r);
}


void CPU::pop(WordRegister r) {
	
	r = memory[SP].read_word();
	SP += 2;
}


void CPU::run() {
	
	while (on) {
		
#ifdef VDEBUG
		std::string opstr = instruction_strings[instruction_set][memory[PC].read_byte()];
		std::stringstream in;
		
		if (opstr.find("xxxx") != opstr.npos) {
			std::stringstream hex;
			hex << to_hex(memory[PC+1].read_word(), 4);
			opstr.replace(opstr.find("xxxx"), 4, hex.str());
		}
		
		if (opstr.find("xx") != opstr.npos) {
			std::stringstream hex;
			hex << to_hex(memory[PC+1].read_byte(), 2);
			opstr.replace(opstr.find("xx"), 2, hex.str());
		}
		printf("%04X %02X %s\n", PC, memory[PC].read_byte(), opstr.c_str());
		assert(memory[PC].read_byte() != 0xFF);
		
		if (PC == 0x0312) throw MemoryException("Breakpoint");
#endif
		clk.cycle_start();
		handle_interrupts();
		
		if (halted) {
			clk += 4;
			continue;
		}
		
		Instruction &instr = instruction_table[instruction_set][memory[PC++].read_byte()];
		instruction_set = 0;
		
		clk += instr.fn(instr.args);
		clk.cycle_end();
		
#ifdef SANITY
		assert(Registers::__ == 0xABCD);
#endif
	}
}
