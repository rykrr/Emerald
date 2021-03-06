#pragma once
#include "memory.hh"


namespace Interrupt {
	
	enum InterruptType : u8 {
		VBLANK	= 0x01,
		LCDSTAT	= 0x02,
		TIMER	= 0x04,
		SERIAL	= 0x08,
		JOYPAD	= 0x10
	};
}


inline void interrupt(Interrupt::InterruptType i) {
	
	memory[0xFF0F].write_byte(memory.read_byte() | i);
}
