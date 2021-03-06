#pragma once
#include "instructions.hh"
#include "registers.hh"
#include "cpu.hh"
#include "../memory.hh"

namespace Instructions {
	
	using namespace Flags;
	using namespace Registers;

	#define instr(fn) inline u8 fn(InstructionArgs args)
	
	#define SET_FLAG(f) F |= f;
	#define CLEAR_FLAG(f) F &= ~f;
	
	#define CHECK_FLAGS										\
		(( ( args.flag & NOT ) && !( F & args.flag ) )		\
		|| ( !( args.flag & NOT ) && ( F & args.flag ) ))
	
	
	#define LOAD_U8_OPERAND_B					\
		u8 b;									\
												\
		if (indirect)							\
			b = memory[args.src16].read_byte();	\
		else if (immediate)						\
			b = memory[PC++].read_byte();		\
		else									\
			b = args.src;
	
	#define LOAD_U8_OPERAND_A							\
		u8 a =											\
			indirect?	memory[args.src16].read_byte()	\
					:	args.dst;
	
	#define STORE_U8_OPERAND_A					\
		if(indirect)							\
			memory[args.src16].write_byte(a);	\
		else									\
			args.dst = a;
	
	#define _return return args.cycles

	#define IND(addr) (memory[addr].read_word())
	#define HIGH 0xFF00
	
	instr(NOP) {
		_return;
	}
	
	
	instr(ILL) {
		throw std::runtime_error("Illegal Instruction");
		_return;
	}
	
	
	instr(HALT) {
		cpu.halt();
		_return;
	}
	
	
	instr(STOP) {
		cpu.stop();
		_return;
	}
	
	
	instr(EI) {
		cpu.enable_interrupts(0, true);
		_return;
	}
	
	
	instr(DI) {
		cpu.disable_interrupts(0, true);
		_return;
	}
	
	
	instr(CB) {
		cpu.cb();
		_return;
	}
	
	
	template<bool immediate> instr(JP) {
		
		if (args.flag != ANY) {
			
			if (!CHECK_FLAGS) {
				if (immediate) PC += 2;
				_return;
			}
			
			args.cycles += 4;
		}
		
		PC = immediate? memory[PC].read_word() : args.src16;
		_return;
	}
	
	
	instr(JR) {
		
		PC++;
		
		if (args.flag != ANY) {
			if (!CHECK_FLAGS) _return;
			args.cycles += 4;
		}
		
		PC += static_cast<i8>(memory[PC-1].read_byte());
		_return;
	}
	
	
	instr(RST) {
		
		cpu.push(PC);
		PC = args.data;
		_return;
	}
	
	
	instr(CALL) {
		
		u16 addr = PC;
		PC += 2;
		
		if (args.flag != ANY) {
			if (!CHECK_FLAGS) _return;
			args.cycles += 12;
		}
		
		cpu.push(PC);
		PC = memory[addr].read_word();
		_return;
	}
	
	
	template<bool int_enable> instr(RET) {
		
		if (args.flag != ANY) {
			if (!CHECK_FLAGS) _return;
			args.cycles += 16;
		}
		
		cpu.pop(PC);
		
		if (int_enable)
			cpu.enable_interrupts(0, true);

		_return;
	}
	
	
	instr(MV) {
		args.dst = args.src;
		_return;
	}
	
	
	template<bool indirect, int difference> instr(LD) {
		
		args.dst = memory[indirect? IND(args.src16): args.src16].read_byte();
		args.src16 += difference;
		_return;
	}
	
	
	template<bool indirect, int difference> instr(ST) {
		
		memory[indirect? IND(args.dst16) : args.dst16].write_byte(args.src);
		args.dst16 += difference;
		_return;
	}
	
	
	template<bool immediate> instr(LDH) {
		
		args.dst = memory[HIGH + (immediate? memory[PC++].read_byte() : args.src)].read_byte();
		_return;
	}
	
	
	template<bool immediate> instr(STH) {
		
		memory[HIGH + (immediate? memory[PC++].read_byte() : args.dst)].write_byte(args.src);
		_return;
	}
	
	
	template<bool indirect> instr(LET) {
		
		if (indirect) {
			u8 val = memory[PC++].read_byte();
			memory[args.dst16].write_byte(val);
			_return;
		}
		
		args.dst = memory[PC++].read_byte();
		_return;
	}
	
	
	instr(MV16) {
		args.dst16 = args.src16;
		_return;
	}
	
	
	instr(LET16) {
		args.dst16 = memory[PC].read_word();
		PC += 2;
		_return;
	}
	
	
	instr(MVSP) {
		HL = SP + memory[PC++].read_byte();
		_return;
	}
	
	
	instr(STSP) {
		
		memory[memory[PC].read_word()].write_word(SP);
		PC += 2;
		_return;
	}
	

	instr(POP) {
		cpu.pop(args.dst16);
		_return;
	}
	
	
	instr(PUSH) {
		cpu.push(args.src16);
		_return;
	}
	

	instr(DAA) {

		if (F & SUBTRACT) {
			if (F & CARRY) A -= 0x60;
			if (F & HALF_CARRY) A -= 0x06;
		}
		else {
			if (F & CARRY || A > 0x99) {
				A += 0x60;
				SET_FLAG(CARRY);
			}
			if (F & HALF_CARRY || (A & 0x0F) > 0x09)
				A += 0x06;
		}

		if (!A) SET_FLAG(ZERO);
		CLEAR_FLAG(HALF_CARRY);
		_return;
	}
	
	
	template<bool carry, bool indirect, bool immediate>	instr(ADD) {
		
		u16 sum = 0;
		u8 half = 0;
		
		LOAD_U8_OPERAND_B;
		
		if (carry && (F & CARRY))
			sum = half = 1;
		
		sum += args.dst + b;
		half += (args.dst & 0xF) + (b & 0xF);
		
		F = 0;
		if (half & 0x10) SET_FLAG(HALF_CARRY);
		if (sum & 0x100) SET_FLAG(CARRY);
		if (!(sum & 0xFF)) SET_FLAG(ZERO);
		
		args.dst = (sum & 0xFF);
		_return;
	}
	

	template<bool carry, bool indirect, bool immediate>	instr(SUB) {

		i16 diff = 0;
		u8 half = 0;
		
		LOAD_U8_OPERAND_B;
		
		if (carry && (F & CARRY))
			diff = half = -1;
		
		diff += args.dst - b;
		half += (args.dst & 0xF0) - (b & 0xF0);
		
		F = SUBTRACT;
		if (half & 0x4) SET_FLAG(HALF_CARRY); // TODO - Review
		if (diff < 0) SET_FLAG(CARRY);
		if (!diff) SET_FLAG(ZERO);
		
		args.dst = (diff & 0xFF);
		_return;
	}
	
	
	template<bool indirect>	instr(INC) {
		
		LOAD_U8_OPERAND_A;
		u8 half = (a & 0xF) + 1;
		
		a++;
		
		F &= CARRY;
		if (half & 0x10) SET_FLAG(HALF_CARRY);
		if (!a) SET_FLAG(ZERO);
		
		STORE_U8_OPERAND_A;
		_return;
	}
	
	
	template<bool indirect>	instr(DEC) {
		
		LOAD_U8_OPERAND_A;
		u8 half = (a & 0x10) - 1;
		
		a--;
		
		F = (F & CARRY) | SUBTRACT;
		if (half & 0x4) SET_FLAG(HALF_CARRY);
		if (!a) SET_FLAG(ZERO);
		
		STORE_U8_OPERAND_A;
		_return;
	}
	
	
	instr(ADD16) {
		
		u16 &a = args.dst16;
		u16 &b = args.src16;
		
		u8 a_hi = a >> 8;
		u8 b_hi = b >> 8;
		
		u16 sum_hi = a_hi + b_hi;
		u8 half_hi = (a_hi & 0xF) + (b_hi & 0xF);
		
		F &= ZERO;
		if (sum_hi & 0x100) SET_FLAG(CARRY);
		if (half_hi & 0x10) SET_FLAG(HALF_CARRY);
		
		a += b;
		
		_return;
	}
	
	
	instr(INC16) {
		args.dst16++;
		_return;
	}
	
	
	instr(DEC16) {
		args.dst16--;
		_return;
	}
	
	
	instr(ADDS) {
		
		u16 a = SP;
		i8 b = static_cast<i8>(memory[PC++].read_byte());
		
		i32 sum = a + b;
		i32 half = ((a >> 8) & ((b < 0) ? 0xF0 : 0xF)) + b; // TODO - Revise b
		
		if (sum < 0) ; // TODO - Throw exception
		else if (sum & 0x10000)
			SET_FLAG(CARRY);
		
		if ((b < 0 && (half & 0x4)) || (b >= 0 && (half & 0x10)))
			SET_FLAG(HALF_CARRY);

		_return;
	}
	
	
	instr(CPL) {
		
		args.dst = ~args.dst;
		F |= SUBTRACT | CARRY;
		_return;
	}
	
	
	template<bool indirect, bool immediate>	instr(AND) {
		
		LOAD_U8_OPERAND_B;
		args.dst &= b;
		
		F = HALF_CARRY;
		if (!args.dst) SET_FLAG(ZERO);
		
		_return;
	}
	
	
	template<bool indirect, bool immediate>	instr(XOR) {
		
		LOAD_U8_OPERAND_B;
		args.dst ^= b;
		
		F = 0;
		if (!args.dst) SET_FLAG(ZERO);
		
		_return;
	}
	
	
	template<bool indirect, bool immediate>	instr(OR) {
		
		LOAD_U8_OPERAND_B;
		args.dst |= b;
		
		F = 0;
		if (!args.dst) SET_FLAG(ZERO);
		
		_return;
	}
	
	
	template<bool indirect, bool immediate>	instr(CP) {
		
		u8 val = args.dst;
		SUB<false, indirect, immediate>(args);
		args.dst = val;
		_return;
	}
	
	
	template<bool carry, bool indirect>	instr(RR) {
		
		LOAD_U8_OPERAND_A;
		u8 original = a;
		
		a >>= 1;
			
		if (carry)
			a |= !!(F & CARRY) << 7;
		else
			a |= (original & 1) << 7;
		
		F = 0;
		if (!a) SET_FLAG(ZERO);
		if (original & 1) SET_FLAG(CARRY);
		
		STORE_U8_OPERAND_A;
		_return;
	}
	
	
	template<bool carry, bool indirect>	instr(RL) {
		
		LOAD_U8_OPERAND_A;
		u8 original = a;
		
		a <<= 1;
			
		if (carry)
			a |= !!(F & CARRY);
		else
			a |= !!(original & 0x80);
		
		F = 0;
		if (!a) SET_FLAG(ZERO);
		if (original & 0x80) SET_FLAG(CARRY);
		
		STORE_U8_OPERAND_A;
		_return;
	}
	
	
	template<bool indirect>	instr(SLA) {
		
		LOAD_U8_OPERAND_A;
		
		F = 0;
		
		if (a & 0x40) SET_FLAG(CARRY);
		a <<= 1;
		if (!a) SET_FLAG(ZERO);
		
		STORE_U8_OPERAND_A;
		_return;
	}
	
	
	template<bool indirect>	instr(SRL) {

		LOAD_U8_OPERAND_A;
		
		F = 0;
		
		if (a & 1) SET_FLAG(CARRY);
		a >>= 1;
		if (!a) SET_FLAG(ZERO);
		
		STORE_U8_OPERAND_A;
		_return;
	}
	
	template<bool indirect>	instr(SRA) {

		LOAD_U8_OPERAND_A;
		
		F = 0;
		if (a & 1) SET_FLAG(CARRY);
		
		a = (a & 0x40) | (a >> 1);
		if (!a) SET_FLAG(ZERO);
		
		STORE_U8_OPERAND_A;
		_return;
	}
	
	
	template<bool indirect>	instr(SWAP) {
		
		LOAD_U8_OPERAND_A;
		
		a = a >> 4 | a << 4;
		
		F = 0;
		if (!a) SET_FLAG(ZERO);
		
		STORE_U8_OPERAND_A;
		_return;
	}
	
	
	template<bool indirect>	instr(BIT) {
		
		LOAD_U8_OPERAND_A;
		
		F &= CARRY;
		SET_FLAG(HALF_CARRY);
		
		if (!(a & (1 << args.data)))
			SET_FLAG(ZERO);
		
		STORE_U8_OPERAND_A;
		_return;
	}
	
	
	template<bool indirect>	instr(SET) {
		
		LOAD_U8_OPERAND_A;
		a |= (1 << args.data);
		STORE_U8_OPERAND_A;
		_return;
	}
	
	
	template<bool indirect>	instr(RES) {
		
		LOAD_U8_OPERAND_A;
		a &= ~(1 << args.data);
		STORE_U8_OPERAND_A;
		_return;
	}
	
	#undef IND
	#undef HIGH
	#undef SET_FLAG
	#undef CLEAR_FLAG
	#undef CHECK_FLAGS
	#undef LOAD_U8_OPERAND_B
	#undef LOAD_U8_REF_OPERAND_A
	#undef _return
	#undef instr
}
