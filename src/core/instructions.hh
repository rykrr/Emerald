#pragma once
#include <string>
#include "registers.hh"


struct InstructionArgs {
	
	ByteRegister	dst;
	ByteRegister	src;
	
	WordRegister	dst16;
	WordRegister	src16;
	
	u8				data;
	Flags::Flags	flag;

	u8				cycles;
};


struct Instruction {
	
	u8 (*fn)(InstructionArgs);
	InstructionArgs	args;
};


namespace Instructions {
	#define instr(fn) u8 fn(InstructionArgs)
	
	instr(NOP);		// No operation
	instr(ND);		// Not defined (exception)
	
	instr(HALT);
	instr(STOP);
	instr(EI);
	instr(DI);
	
	instr(CB);
	
	template<bool immediate>	instr(JP);
								instr(JR);
								instr(RST);
								instr(CALL);
	template<bool int_enable>	instr(RET);
	
											instr(MV);
	template<bool indirect, int difference>	instr(LD);
	template<bool indirect, int difference>	instr(ST);
	
	template<bool immediate>	instr(LDH);
	template<bool immediate>	instr(STH);
	
	template<bool indirect>		instr(LET);
	
	instr(MV16);
	instr(LET16);
	
	instr(MVSP);	// ld hl, sp+i8
	instr(STSP);	// ld (a16), sp
	
	instr(POP);
	instr(PUSH);
	
	instr(DAA);
	
	template<bool carry, bool indirect, bool immediate>	instr(ADD);	
	template<bool carry, bool indirect, bool immediate>	instr(SUB);
	
	template<bool indirect>	instr(INC);
	template<bool indirect>	instr(DEC);
	
	instr(ADD16);
	instr(INC16);
	instr(DEC16);
	
	instr(ADDS);
	
											instr(CPL);
	template<bool indirect, bool immediate>	instr(AND);
	template<bool indirect, bool immediate>	instr(XOR);
	template<bool indirect, bool immediate>	instr(OR);
	template<bool indirect, bool immediate>	instr(CP);
	
	template<bool carry, bool indirect>	instr(RR);
	template<bool carry, bool indirect>	instr(RL);
	
	template<bool indirect>	instr(SLA);
	template<bool indirect>	instr(SRL);
	template<bool indirect>	instr(SRA);
	
	template<bool indirect>	instr(SWAP);
	
	template<bool indirect>	instr(BIT);
	template<bool indirect>	instr(SET);
	template<bool indirect>	instr(RES);
	
	#undef instr
}

#include "instructions.tcc"

