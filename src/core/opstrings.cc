#include "mappings.hh"

#define PATTERN(op, y) op ", " y

#define REPEAT_PATTERN(op) \
	PATTERN(op, "b"),	PATTERN(op, "c"),	PATTERN(op, "d"),		PATTERN(op, "e"),\
	PATTERN(op, "h"),	PATTERN(op, "l"),	PATTERN(op, "(hl)"),	PATTERN(op, "a")

#define REPEAT_PATTERN_ALT7(op, op7str) \
	PATTERN(op, "b"),	PATTERN(op, "c"),	PATTERN(op, "d"),	PATTERN(op, "e"),\
	PATTERN(op, "h"),	PATTERN(op, "l"),	op7str,				PATTERN(op, "a")


std::string instruction_strings[2][256] = {{
	"nop",
	"ld bc, xxxx",
	"ld (bc), a",
	"inc bc",
	"inc b",
	"dec b",
	"ld b, xx",
	"rlca",
	
	"ld (xxxx), sp",
	"add hl, bc",
	"ld a, (bc)",
	"dec bc",
	"inc c",
	"dec c",
	"ld c, xx",
	"rrca",
	
	"stop",
	"ld de, xxxx",
	"ld (de), a",
	"inc de",
	"inc d",
	"dec d",
	"ld d, xx",
	"rla",
	
	"jr xx",
	"add hl, de",
	"ld a, (de)",
	"dec de",
	"inc e",
	"dec e",
	"ld e, xx",
	"rra",
	
	"jr nz, xx",
	"ld hl, xxxx",
	"ld (hl+), a",
	"inc hl",
	"inc h",
	"dec h",
	"ld h, xx",
	"daa",
	
	"jr z, xx",
	"add hl, hl",
	"ld a, (hl+)",
	"dec hl",
	"inc l",
	"dec l",
	"ld l, xx",
	"cpl",
	
	"jr nc, xx",
	"ld sp, xxxx",
	"ld (hl-), a",
	"inc sp",
	"inc (hl)",
	"dec (hl)",
	"ld (hl), xx",
	"scf",
	
	"jr c, xx",
	"add hl, sp",
	"ld a, (hl-)",
	"dec sp",
	"inc a",
	"dec a",
	"ld a, xx",
	"ccf",
	
	REPEAT_PATTERN("ld b"),					REPEAT_PATTERN("ld c"),
	REPEAT_PATTERN("ld d"),					REPEAT_PATTERN("ld e"),
	REPEAT_PATTERN("ld h"),					REPEAT_PATTERN("ld l"),
	REPEAT_PATTERN_ALT7("ld (hl)", "halt"),	REPEAT_PATTERN("ld a"),
	REPEAT_PATTERN("add a"),				REPEAT_PATTERN("adc a"),
	REPEAT_PATTERN("sub a"),				REPEAT_PATTERN("sbc a"),
	REPEAT_PATTERN("and a"),				REPEAT_PATTERN("xor a"),
	REPEAT_PATTERN("or a"),					REPEAT_PATTERN("cp a"),
	
	"ret nz",
	"pop bc",
	"jp nz, xxxx",
	"jp xxxx",
	"call nz, xxxx",
	"push bc",
	"add a, xx",
	"rst 00h",
	
	"ret z",
	"ret",
	"jp z, xxxx",
	"cb",
	"call z, xxxx",
	"call xxxx",
	"adc a, xx",
	"rst 08h",
	
	"ret nc",
	"pop de",
	"jp nc, xxxx",
	"nop d3",
	"call nc, xxxx",
	"push de",
	"sub xx",
	"rst 10h",
	
	"ret c",
	"reti",
	"jp c, xxxx",
	"nop db",
	"call c, xxxx",
	"nop dd",
	"sbc a, xx",
	"rst 18h",
	
	"ldh (xx), a",
	"pop hl",
	"ld (c), a",
	"nop e3",
	"nop e4",
	"push hl",
	"and xx",
	"rst 20h",
	
	"add sp, xx",
	"jp (hl)",
	"ld (xxxx), a",
	"nop eb",
	"nop ec",
	"nop ed",
	"xor xx",
	"rst 28h",
	
	"ldh a, (xx)",
	"pop af",
	"ld a, (c)",
	"di",
	"nop f4",
	"push af",
	"or xx",
	"rst 30h",
	
	"ld hl, sp+xx",
	"ld sp, hl",
	"ld a, (xxxx)",
	"ei",
	"nop fb",
	"nop fc",
	"cp xx",
	"rst 38h",
}, {
	REPEAT_PATTERN("rlc"),		REPEAT_PATTERN("rrc"),
	REPEAT_PATTERN("rl"),		REPEAT_PATTERN("rr"),
	REPEAT_PATTERN("sla"),		REPEAT_PATTERN("sra"),
	REPEAT_PATTERN("swap"),		REPEAT_PATTERN("srl"),
	REPEAT_PATTERN("bit 0"),	REPEAT_PATTERN("bit 1"),
	REPEAT_PATTERN("bit 2"),	REPEAT_PATTERN("bit 3"),
	REPEAT_PATTERN("bit 4"),	REPEAT_PATTERN("bit 5"),
	REPEAT_PATTERN("bit 6"),	REPEAT_PATTERN("bit 7"),
	REPEAT_PATTERN("res 0"),	REPEAT_PATTERN("res 1"),
	REPEAT_PATTERN("res 2"),	REPEAT_PATTERN("res 3"),
	REPEAT_PATTERN("res 4"),	REPEAT_PATTERN("res 5"),
	REPEAT_PATTERN("res 6"),	REPEAT_PATTERN("res 7"),
	REPEAT_PATTERN("set 0"),	REPEAT_PATTERN("set 1"),
	REPEAT_PATTERN("set 2"),	REPEAT_PATTERN("set 3"),
	REPEAT_PATTERN("set 4"),	REPEAT_PATTERN("set 5"),
	REPEAT_PATTERN("set 6"),	REPEAT_PATTERN("set 7")
}};

#undef PATTERN
#undef REPEAT_PATTERN
#undef REPEAT_PATTERN_ALT7
