#pragma once
#include <cstdio>
#include <iomanip>

#define to_hex(x, b)			\
	std::uppercase				\
		<< std::hex				\
		<< std::setw(b)			\
		<< std::setfill('0')	\
		<< static_cast<u16>(x)

#ifdef VDEBUG
#define dprintf(args...) printf(args)
#else
#define dprintf(args...)
#endif

#undef assert
#define assert(assertion) \
	if (!(assertion)) throw std::runtime_error("Assertion failed: " #assertion);

#define bind(variable, haddr) variable(memory[0xFF##haddr].byte())

#define unused(x) (void)(x)
