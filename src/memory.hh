#pragma once

#include <iostream>

#include <functional>
#include <stdexcept>

#include "macros.hh"


#define ROM_FLAG
#define IO_REGISTER_MASK 0xFF80

typedef std::runtime_error MemoryException;
typedef std::function<u8(u16, u8, bool)> MemoryCallback;

#define cmem_lambda(a, b, w) [this](u16 a, u8 b, bool w) -> u8

class Memory {
	
	u8 memory[0x10000];
	
	MemoryCallback bankctl;
	
	struct {
		bool initialized;
		bool callable;
		MemoryCallback callback;
		u8 *ptr;
	} registers[0x81], *_register;
	
	u16 addr;
	
	bool debug;
	
public:

	void init_register(u8 r, u8 &ptr) {
		
		if (r >= 0x81) throw MemoryException("Invalid register number");
		registers[r].initialized = true;
		registers[r].ptr = &ptr;
	}
	
	void init_register(u8 r, MemoryCallback callback) {
		
		if (r >= 0x81) throw MemoryException("Invalid register number");
		registers[r].initialized = true;
		registers[r].callback = callback;
		registers[r].callable = true;
	}
	
	Memory &operator[](u16 addr) {
		
		assert(addr != 0xABCD);
		this->addr = addr;
		
		_register =
			(addr & 0xFF80) == 0xFF00
				? &registers[addr & 0xFF]
				: (addr == 0xFFFF)
					? &registers[0x80]
					: NULL;
		
		return *this;
	}
	
	u8 read_byte() {
		
		if (!_register) return memory[addr];
		if (!_register->initialized) return 0xFF;
		return _register->callable? _register->callback(addr, 0, false) : *_register->ptr;
	}
	
	u16 read_word() {
	
		if (_register) throw MemoryException("Illegal word operation on byte register");
		return *((u16*) &memory[addr]);
	}
	
	void write_byte(u8 b) {
		
		if (_register) {
			if (!_register->initialized) return;
			
			if (_register->callable)
				_register->callback(addr, b, true);
			else
				*_register->ptr = b;
			
			return;
		}
		
		if (addr & 0x8000) memory[addr] = b;
		else bankctl(addr, b, true);
	}
	
	void write_word(u16 w) {
		if (_register) throw MemoryException("Illegal word operation on byte register");
		if (debug || addr & 0x8000) *((u16*) &memory[addr]) = w;
		else throw MemoryException("Illegal write to rom");
	}

	void copy(u16 dst_addr, u16 src_addr) {
		memory[dst_addr] = memory[src_addr];
	}
	
	void copy(u16 dst_addr, u8 *src, u16 len) {
		
		if (dst_addr + len >= 0xFF00)
			throw MemoryException("Address range of copy() overlaps with io registers");
		
		for (u16 i = 0; i < len; i++)
			memory[dst_addr + i] = src[i];
	}
	
	void set_bank_controller(MemoryCallback bankctl) {
		
		this->bankctl = bankctl;
	}

	void set_debug_mode(bool mode) {
		debug = mode;
	}
	
public:
	
	Memory(): debug(false) {
		
		for (u8 r = 0; r < 0x80; r++)
			registers[r] = {};
		
		_register = NULL;
		
		bankctl = [this](u16 a, u8 b, bool w) -> u8 {
			throw MemoryException("Illegal write to ROM");
		};
	}
};

extern Memory memory;
