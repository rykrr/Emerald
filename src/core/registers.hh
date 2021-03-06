#pragma once
#include <cstdint>

typedef u8		&ByteRegister;
typedef u16	&WordRegister;


namespace Flags {
	
	enum Flags : u8 {
		
		CARRY		= 0x10,
		HALF_CARRY	= 0x20,
//		HALF_CARRY	= 0x00,
		SUBTRACT	= 0x40,
		ZERO		= 0x80,

		ANY			= 0x00,
		NOT			= 0x01,
		NONZERO		= 0x81,
		NOCARRY		= 0x11
	};
}

#define PAIR(hi, lo)		\
	union {					\
		u16 hi##lo;	\
		struct {			\
			u8 lo;		\
			u8 hi;		\
		};					\
	}


struct RegisterFile {
	
	PAIR(A, F);
	PAIR(B, C);
	PAIR(D, E);
	PAIR(H, L);
	u16 SP;
	u16 PC;
};

#undef PAIR
#define PAIR(hi, lo)			\
	extern ByteRegister hi, lo;	\
	extern WordRegister hi##lo;

namespace Registers {

	PAIR(A, F);
	PAIR(B, C);
	PAIR(D, E);
	PAIR(H, L);
	extern WordRegister SP;
	extern WordRegister PC;

	// Special placeholder registers
	extern ByteRegister _;
	extern WordRegister __;
}

#undef PAIR

extern RegisterFile registers;
