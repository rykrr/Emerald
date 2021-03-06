#pragma once
#include "../clock.hh"
#include "../memory.hh"
#include "../interrupt.hh"
#include "../macros.hh"

#define FALLING_EDGE(x) (!( clock & x ) && ( clock_prev & x ))

class Timer : public ClockSubscriber {
	
	union {
		struct {
			u8 : 8; // Unused clock_lo field
			u8 clock_hi;
		};
		u16 clock;
	};

	u16 clock_prev;
	
	u8 DIV;	// Divider Register (increment 16384Hz or 32768Hz)
	u8 TIMA;	// Timer Counter
	u8 TMA;	// Timer Modulo
	u8 TAC;	// Timer Control 
	
public:
	
	void operator+=(u8 cycles) {
		
		// if DIV has been overwritten
		if (DIV != clock_hi) {
			DIV = clock = clock_prev = 0;
			return;
		}
		
		clock_prev = clock;
		clock += cycles;
		DIV = clock_hi;
		
		if (FALLING_EDGE(0x2000))
			; // TODO - Sound clock trigger
		
		// if Timer disabled
		if (!(TAC & 4))
			return;
		
		switch (TAC & 3) {
			case 0: if (FALLING_EDGE(1024)) return; break;
			case 1: if (FALLING_EDGE(16))   return; break;
			case 2: if (FALLING_EDGE(64))   return; break;
			case 3: if (FALLING_EDGE(256))  return; break;
		}
		
		// if Timer does not overflow
		if (TIMA++)
			return;
		
		TIMA = TMA;
		interrupt(Interrupt::TIMER);
	}
	
	inline u16 get_clock_value() { return clock; }
	
	inline void enable() { TAC |= 4; }
	inline void disable() { TAC &= ~4; }
	
	
public:
	
	Timer()
		:	clock(0),	clock_prev(0),
			DIV(0),		TIMA(0),
			TMA(0),		TAC(0)	{

		memory.init_register(0x04, DIV);
		memory.init_register(0x05, TIMA);
		memory.init_register(0x06, TMA);
		memory.init_register(0x07, TAC);
	}
};

#undef FALLING_EDGE

extern Timer timer;
