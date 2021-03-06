#include "mappings.hh"
#include "registers.hh"

using namespace Flags;
using namespace Registers;
using namespace Instructions;


#define REGISTER(x, y)						{ x, y, __, __, 0x00, ANY, 0x04 }
#define INDIRECT(x, y)						{ x, _, __, y,  0x00, ANY, 0x08 }

#define _HALT(...)	{ HALT				,	{ _, _, __, __, 0x00, ANY, 0x04 }	}
	
#define _MV(x, y)	{ MV				,	REGISTER(x, y)						}
#define _LD(x, y)	{ LD	<0, 0>		,	INDIRECT(x, y)						}
#define _ST(x, y)	{ ST	<0, 0>		,	{ _, y, x,  __, 0x00, ANY ,	0x08 }	}

// ADD < CARRY, INDIRECT, IMMEDIATE >
#define _ADD(x, y)	{ ADD	<0, 0, 0>	,	REGISTER(x, y)						}
#define _ADDi(x, y)	{ ADD	<0, 1, 0>	,	INDIRECT(x, y)						}

#define _ADC(x, y)	{ ADD	<1, 0, 0>	,	REGISTER(x, y)						}
#define _ADCi(x, y)	{ ADD	<1, 1, 0>	,	INDIRECT(x, y)						}

// SUB < CARRY, INDIRECT, IMMEDIATE >
#define _SUB(x, y)	{ SUB	<0, 0, 0>	,	REGISTER(x, y)						}
#define _SUBi(x, y)	{ SUB	<0, 1, 0>	,	INDIRECT(x, y)						}
#define _SBC(x, y)	{ SUB	<1, 0, 0>	,	REGISTER(x, y)						}
#define _SBCi(x, y)	{ SUB	<1, 0, 0>	,	INDIRECT(x, y)						}

#define _AND(x, y)	{ AND	<0, 0>		,	REGISTER(x, y)						}
#define _ANDi(x, y)	{ AND	<1, 0>		,	INDIRECT(x, y)						}
#define _XOR(x, y)	{ XOR	<0, 0>		,	REGISTER(x, y)						}
#define _XORi(x, y)	{ XOR	<1, 0>		,	INDIRECT(x, y)						}
#define _OR(x, y)	{ OR	<0, 0>		,	REGISTER(x, y)						}
#define _ORi(x, y)	{ OR	<1, 0>		,	INDIRECT(x, y)						}
#define _CP(x, y)	{ CP	<0, 0>		,	REGISTER(x, y)						}
#define _CPi(x, y)	{ CP	<1, 0>		,	INDIRECT(x, y)						}

#define _RL(x, y)	{ RL	<x, 0>		,	{ y, _, __, __, 0x00, ANY, 0x08 }	}
#define _RLi(x, y)	{ RL	<x, 1>		,	{ _, _, __, y,  0x00, ANY, 0x10 }	}
#define _RR(x, y)	{ RR	<x, 0>		,	{ y, _, __, __, 0x00, ANY, 0x08 }	}
#define _RRi(x, y)	{ RR	<x, 1>		,	{ _, _, __, y,  0x00, ANY, 0x10 }	}
#define _SL(x, y)	{ SLA	<0>			,	{ y, _, __, __, 0x00, ANY, 0x08 }	}
#define _SLi(x, y)	{ SLA	<1>			,	{ _, _, __, y,  0x00, ANY, 0x10 }	}
#define _SRA(x, y)	{ SRA	<0>			,	{ y, _, __, __, 0x00, ANY, 0x08 }	}
#define _SRAi(x, y)	{ SRA	<1>			,	{ _, _, __, y,  0x00, ANY, 0x10 }	}
#define _SRL(x, y)	{ SRL	<0>			,	{ y, _, __, __, 0x00, ANY, 0x08 }	}
#define _SRLi(x, y)	{ SRL	<1>			,	{ _, _, __, y,  0x00, ANY, 0x10 }	}
                                                                             	 
#define _SWAP(x, y)	{ SWAP	<0>			,	{ y, _, __, __, 0x00, ANY, 0x08 }	}
#define _SWAPi(x, y){ SWAP	<1>			,	{ _, _, __, y,  0x00, ANY, 0x10 }	}
                                                                             	 
#define _BIT(x, y)	{ BIT	<0>			,	{ y, _, __, __, x,    ANY, 0x08 }	}
#define _BITi(x, y)	{ BIT	<1>			,	{ _, _, __, y,  x,    ANY, 0x10 }	}
                                                                             	 
#define _RES(x, y)	{ RES	<0>			,	{ y, _, __, __, x,    ANY, 0x08 }	}
#define _RESi(x, y)	{ RES	<1>			,	{ _, _, __, y,  x,    ANY, 0x10 }	}
                                                                             	 
#define _SET(x, y)	{ SET	<0>			,	{ y, _, __, __, x,    ANY, 0x08 }	}
#define _SETi(x, y)	{ SET	<1>			,	{ _, _, __, y,  x,    ANY, 0x10 }	}


#define REPEAT(pattern, pattern7, arg)	\
	pattern(arg, B),	pattern(arg, C),	pattern(arg, D),	pattern(arg, E),\
	pattern(arg, H),	pattern(arg, L),	pattern7(arg, HL),	pattern(arg, A)


Instruction instruction_table[2][256] = {{
	
	{	NOP					,	{ _, _, __, __, 0x00, ANY		,	0x04	}},
	{	LET16				,	{ _, _, BC, __, 0x00, ANY		,	0x0C	}},
	{	ST		<0, 0>		,	{ _, A, BC, __, 0x00, ANY		,	0x08	}},
	{	INC16				,	{ _, _, BC, __, 0x00, ANY		,	0x08	}},
	{	INC		<0>			,	{ B, _, __, __, 0x00, ANY		,	0x04	}},
	{	DEC		<0>			,	{ B, _, __, __, 0x00, ANY		,	0x04	}},
	{	LET		<0>			,	{ B, _, __, __, 0x00, ANY		,	0x08	}},
	{	RL		<0, 0>		,	{ A, _, __, __, 0x00, ANY		,	0x04	}},	// RLCA
	
	{	STSP				,	{ _, _, __, __, 0x00, ANY		,	0x14	}},
	{	ADD16				,	{ _, _, HL, BC, 0x00, ANY		,	0x0C	}},
	{	LD		<0, 0>		,	{ A, _, __, BC, 0x00, ANY		,	0x08	}},
	{	DEC16				,	{ _, _, BC, __, 0x00, ANY		,	0x08	}},
	{	INC		<0>			,	{ C, _, __, __, 0x00, ANY		,	0x04	}},
	{	DEC		<0>			,	{ C, _, __, __, 0x00, ANY		,	0x04	}},
	{	LET		<0>			,	{ C, _, __, __, 0x00, ANY		,	0x08	}},
	{	RR		<0, 0>		,	{ A, _, __, __, 0x00, ANY		,	0x04	}}, // RRCA
	
	{	STOP				,	{ _, _, __, __, 0x00, ANY		,	0x04	}},
	{	LET16				,	{ _, _, DE, __, 0x00, ANY		,	0x0C	}},
	{	ST		<0, 0>		,	{ _, A, DE, __, 0x00, ANY		,	0x08	}},
	{	INC16				,	{ _, _, DE, __, 0x00, ANY		,	0x08	}},
	{	INC		<0>			,	{ D, _, __, __, 0x00, ANY		,	0x04	}},
	{	DEC		<0>			,	{ D, _, __, __, 0x00, ANY		,	0x04	}},
	{	LET		<0>			,	{ D, _, __, __, 0x00, ANY		,	0x08	}},
	{	RL		<1, 0>		,	{ A, _, __, __, 0x00, ANY		,	0x04	}}, // RLA
	
	{	JR					,	{ _, _, __, __, 0x00, ANY		,	0x0C	}},
	{	ADD16				,	{ _, _, HL, DE, 0x00, ANY		,	0x0C	}},
	{	LD		<0, 0>		,	{ A, _, __, DE, 0x00, ANY		,	0x08	}},
	{	DEC16				,	{ _, _, DE, __, 0x00, ANY		,	0x08	}},
	{	INC		<0>			,	{ E, _, __, __, 0x00, ANY		,	0x04	}},
	{	DEC		<0>			,	{ E, _, __, __, 0x00, ANY		,	0x04	}},
	{	LET		<0>			,	{ E, _, __, __, 0x00, ANY		,	0x08	}},
	{	RR		<1, 0>		,	{ A, _, __, __, 0x00, ANY		,	0x04	}}, // RRA
	
	{	JR					,	{ _, _, __, __, 0x00, NONZERO	,	0x08	}},
	{	LET16				,	{ _, _, HL, __, 0x00, ANY		,	0x0C	}},
	{	ST		<0, 1>		,	{ _, A, HL, __, 0x00, ANY		,	0x08	}},
	{	INC16				,	{ _, _, HL, __, 0x00, ANY		,	0x08	}},
	{	INC		<0>			,	{ H, _, __, __, 0x00, ANY		,	0x04	}},
	{	DEC		<0>			,	{ H, _, __, __, 0x00, ANY		,	0x04	}},
	{	LET		<0>			,	{ H, _, __, __, 0x00, ANY		,	0x08	}},
	{	DAA					,	{ A, _, __, __, 0x00, ANY		,	0x04	}},
	
	{	JR					,	{ _, _, __, __, 0x00, ZERO		,	0x08	}},
	{	ADD16				,	{ _, _, HL, HL, 0x00, ANY		,	0x0C	}},
	{	LD		<0, 1>		,	{ A, _, __, HL, 0x00, ANY		,	0x08	}},
	{	DEC16				,	{ _, _, HL, __, 0x00, ANY		,	0x08	}},
	{	INC		<0>			,	{ L, _, __, __, 0x00, ANY		,	0x04	}},
	{	DEC		<0>			,	{ L, _, __, __, 0x00, ANY		,	0x04	}},
	{	LET		<0>			,	{ L, _, __, __, 0x00, ANY		,	0x08	}},
	{	CPL					,	{ A, _, __, __, 0x00, ANY		,	0x04	}},
	
	{	JR					,	{ _, _, __, __, 0x00, NOCARRY	,	0x08	}},
	{	LET16				,	{ _, _, SP, __, 0x00, ANY		,	0x0C	}},
	{	ST		<0, -1>		,	{ _, A, HL, __, 0x00, ANY		,	0x08	}},
	{	INC16				,	{ _, _, SP, __, 0x00, ANY		,	0x08	}},
	{	INC		<1>			,	{ _, _, __, HL, 0x00, ANY		,	0x0C	}},
	{	DEC		<1>			,	{ _, _, __, HL, 0x00, ANY		,	0x0C	}},
	{	LET		<1>			,	{ _, _, HL, __, 0x00, ANY		,	0x0C	}},
	{	SET		<0>			,	{ F, _, __, __, 0x04, ANY		,	0x04	}},
	
	{	JR					,	{ _, _, __, __, 0x00, CARRY		,	0x08	}},
	{	ADD16				,	{ _, _, HL, SP, 0x00, ANY		,	0x0C	}},
	{	LD		<0, -1>		,	{ A, _, __, HL, 0x00, ANY		,	0x08	}},
	{	DEC16				,	{ _, _, SP, __, 0x00, ANY		,	0x08	}},
	{	INC		<0>			,	{ A, _, __, __, 0x00, ANY		,	0x04	}},
	{	DEC		<0>			,	{ A, _, __, __, 0x00, ANY		,	0x04	}},
	{	LET		<0>			,	{ A, _, __, __, 0x00, ANY		,	0x08	}},
	{	RES		<0>			,	{ F, _, __, __, 0x04, ANY		,	0x04	}},
	
	REPEAT(	_MV,	_LD,	B	),	REPEAT(	_MV,	_LD,	C	),
	REPEAT(	_MV,	_LD,	D	),	REPEAT(	_MV,	_LD,	E	),
	REPEAT(	_MV,	_LD,	H	),	REPEAT(	_MV,	_LD,	L	),
	REPEAT(	_ST,	_HALT,	HL	),	REPEAT(	_MV,	_LD,	A	),

	REPEAT(	_ADD,	_ADDi,	A	),	REPEAT(	_ADC,	_ADCi,	A	),
	REPEAT(	_SUB,	_SUBi,	A	),	REPEAT(	_SBC,	_SBCi,	A	),
	REPEAT(	_AND,	_ANDi,	A	),	REPEAT(	_XOR,	_XORi,	A	),
	REPEAT(	_OR,	_ORi,	A	),	REPEAT(	_CP,	_CPi,	A	),
	
	{	RET		<0>			,	{ _, _, __, __, 0x00, NONZERO	,	0x08	}},
	{	POP					,	{ _, _, BC, __, 0x00, ANY		,	0x0C	}},
	{	JP		<1>			,	{ _, _, __, __, 0x00, NONZERO	,	0x0C	}},
	{	JP		<1>			,	{ _, _, __, __, 0x00, ANY		,	0x10	}},
	{	CALL				,	{ _, _, __, __, 0x00, NONZERO	,	0x0C	}},
	{	PUSH				,	{ _, _, __, BC, 0x00, ANY		,	0x10	}},
	{	ADD		<0, 0, 1>	,	{ A, _, __, __, 0x00, ANY		,	0x08	}},
	{	RST					,	{ _, _, __, __, 0x00, ANY		,	0x10	}},
	
	{	RET		<0>			,	{ _, _, __, __, 0x00, ZERO		,	0x08	}},
	{	RET		<0>			,	{ _, _, __, __, 0x00, ANY		,	0x10	}},
	{	JP		<1>			,	{ _, _, __, __, 0x00, ZERO		,	0x0C	}},
	{	CB					,	{ _, _, __, __, 0x00, ANY		,	0x00	}},
	{	CALL				,	{ _, _, __, __, 0x00, ZERO		,	0x0C	}},
	{	CALL				,	{ _, _, __, __, 0x00, ANY		,	0x18	}},
	{	ADD		<1, 0, 1>	,	{ A, _, __, __, 0x00, ANY		,	0x08	}},
	{	RST					,	{ _, _, __, __, 0x08, ANY		,	0x10	}},
	
	
	{	RET		<0>			,	{ _, _, __, __, 0x00, NOCARRY	,	0x08	}},
	{	POP					,	{ _, _, DE, __, 0x00, ANY		,	0x0C	}},
	{	JP		<1>			,	{ _, _, __, __, 0x00, NOCARRY	,	0x0C	}},
	{	ILL					,	{ _, _, __, __, 0x00, ANY		,	0x04	}},
	{	CALL				,	{ _, _, __, __, 0x00, NOCARRY	,	0x0C	}},
	{	PUSH				,	{ _, _, __, DE, 0x00, ANY		,	0x10	}},
	{	SUB		<0, 0, 1>	,	{ A, _, __, __, 0x00, ANY		,	0x08	}},
	{	RST					,	{ _, _, __, __, 0x10, ANY		,	0x10	}},
	
	{	RET		<0>			,	{ _, _, __, __, 0x00, CARRY		,	0x08	}},
	{	RET		<1>			,	{ _, _, __, __, 0x00, ANY		,	0x10	}},
	{	JP		<1>			,	{ _, _, __, __, 0x00, CARRY		,	0x0C	}},
	{	ILL					,	{ _, _, __, __, 0x00, ANY		,	0x04	}},
	{	CALL				,	{ _, _, __, __, 0x00, CARRY		,	0x0C	}},
	{	ILL					,	{ _, _, __, __, 0x00, ANY		,	0x04	}},
	{	SUB		<1, 0, 1>	,	{ A, _, __, __, 0x00, ANY		,	0x08	}},
	{	RST					,	{ _, _, __, __, 0x18, ANY		,	0x10	}},
	
	
	{	STH		<1>			,	{ _, A, __, __, 0x00, ANY		,	0x0C	}},
	{	POP					,	{ _, _, HL, __, 0x00, ANY		,	0x0C	}},
	{	STH		<0>			,	{ C, A, __, __, 0x00, ANY		,	0x08	}},
	{	ILL					,	{ _, _, __, __, 0x00, ANY		,	0x04	}},
	{	ILL					,	{ _, _, __, __, 0x00, ANY		,	0x04	}},
	{	PUSH				,	{ _, _, __, HL, 0x00, ANY		,	0x10	}},
	{	AND		<0, 1>		,	{ A, _, __, __, 0x00, ANY		,	0x08	}},
	{	RST					,	{ _, _, __, __, 0x20, ANY		,	0x10	}},
	
	{	ADDS				,	{ _, _, SP, __, 0x00, ANY		,	0x10	}},
	{	JP		<0>			,	{ _, _, __, HL, 0x00, ANY		,	0x0C	}},
	{	ST		<1, 2>		,	{ _, A, PC, __, 0x00, ANY		,	0x10	}},
	{	ILL					,	{ _, _, __, __, 0x00, ANY		,	0x04	}},
	{	ILL					,	{ _, _, __, __, 0x00, ANY		,	0x04	}},
	{	ILL					,	{ _, _, __, __, 0x00, ANY		,	0x04	}},
	{	XOR		<0, 1>		,	{ A, _, __, __, 0x00, ANY		,	0x08	}},
	{	RST					,	{ _, _, __, __, 0x28, ANY		,	0x10	}},
	
	
	{	LDH		<1>			,	{ A, _, __, __, 0x00, ANY		,	0x0C	}},
	{	POP					,	{ _, _, AF, __, 0x00, ANY		,	0x0C	}},
	{	LDH		<0>			,	{ A, C, __, __, 0x00, ANY		,	0x08	}},
	{	DI					,	{ _, _, __, __, 0x00, ANY		,	0x04	}},
	{	ILL					,	{ _, _, __, __, 0x00, ANY		,	0x04	}},
	{	PUSH				,	{ _, _, __, AF, 0x00, ANY		,	0x10	}},
	{	OR		<0, 1>		,	{ A, _, __, __, 0x00, ANY		,	0x08	}},
	{	RST					,	{ _, _, __, __, 0x30, ANY		,	0x10	}},
	
	{	MVSP				,	{ _, _, SP, __, 0x00, ANY		,	0x0C	}},
	{	MV16				,	{ _, _, SP, HL, 0x00, ANY		,	0x08	}},
	{	LD		<1, 2>		,	{ A, _, __, PC, 0x00, ANY		,	0x10	}},
	{	EI					,	{ _, _, __, __, 0x00, ANY		,	0x04	}},
	{	ILL					,	{ _, _, __, __, 0x00, ANY		,	0x04	}},
	{	ILL					,	{ _, _, __, __, 0x00, ANY		,	0x04	}},
	{	CP		<0, 1>		,	{ A, _, __, __, 0x00, ANY		,	0x08	}},
	{	RST					,	{ _, _, __, __, 0x38, ANY		,	0x10	}},
}, {
	REPEAT(	_RL,	_RLi,	0),	REPEAT(	_RR,	_RRi,	0),
	REPEAT(	_RL,	_RLi,	1),	REPEAT(	_RR,	_RRi,	1),
	
	REPEAT(	_SL,	_SLi,	0),	REPEAT(	_SRA,	_SRAi,	0),
	REPEAT(	_SWAP,	_SWAPi,	0),	REPEAT(	_SRL,	_SRLi,	0),
	
	REPEAT(	_BIT,	_BITi,	0),	REPEAT(	_BIT,	_BITi,	1),
	REPEAT(	_BIT,	_BITi,	2),	REPEAT(	_BIT,	_BITi,	3),
	REPEAT(	_BIT,	_BITi,	4),	REPEAT(	_BIT,	_BITi,	5),
	REPEAT(	_BIT,	_BITi,	6),	REPEAT(	_BIT,	_BITi,	7),
	
	REPEAT(	_RES,	_RESi,	0),	REPEAT(	_RES,	_RESi,	1),
	REPEAT(	_RES,	_RESi,	2),	REPEAT(	_RES,	_RESi,	3),
	REPEAT(	_RES,	_RESi,	4),	REPEAT(	_RES,	_RESi,	5),
	REPEAT(	_RES,	_RESi,	6),	REPEAT(	_RES,	_RESi,	7),
	
	REPEAT(	_SET,	_SETi,	0),	REPEAT(	_SET,	_SETi,	1),
	REPEAT(	_SET,	_SETi,	2),	REPEAT(	_SET,	_SETi,	3),
	REPEAT(	_SET,	_SETi,	4),	REPEAT(	_SET,	_SETi,	5),
	REPEAT(	_SET,	_SETi,	6),	REPEAT(	_SET,	_SETi,	7)
}};

#undef REGISTER   
#undef INDIRECT   
#undef _HALT
#undef _MV
#undef _LD
#undef _ST
#undef _ADD
#undef _ADDi
#undef _ADC
#undef _ADCi
#undef _SUB
#undef _SUBi
#undef _SBC
#undef _SBCi

#undef _AND
#undef _ANDi
#undef _XOR
#undef _XORi
#undef _OR
#undef _ORi
#undef _CP
#undef _CPi

#undef _RL
#undef _RLi
#undef _RR
#undef _RRi
#undef _SL
#undef _SLi
#undef _SRA
#undef _SRAi
#undef _SRL
#undef _SRLi

#undef _SWAP
#undef _SWAPi

#undef _BIT
#undef _BITi

#undef _RES
#undef _RESi

#undef _SET
#undef _SETi
#undef REPEAT
