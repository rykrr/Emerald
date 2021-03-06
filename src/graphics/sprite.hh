#pragma once

#define OAM_ATTR_PRI		0x80	// BG Priority (0 = Sprite Over BG)
#define OAM_ATTR_FLY		0x40	// Flip Y
#define OAM_ATTR_FLX		0x20	// Flip X
#define OAM_ATTR_DMG_OBP	0x10	// DMG Object Palette Selector
#define OAM_ATTR_CGB_BNK	0x08	// 
#define OAM_ATTR_CGB_OBP	0x07	// CGB Object Palette Selector

#define OAM_SET_ATTR_PRI(e,p)		(e->attr = (p&1) << 7)
#define OAM_SET_ATTR_FLY(e,f)		(e->attr = (f&1) << 6)
#define OAM_SET_ATTR_FLX(e,f)		(e->attr = (f&1) << 5)
#define OAM_SET_ATTR_DMG_OBP(e,o)	(e->attr = (o&1) << 4)
#define OAM_SET_ATTR_CGB_OBP(e,o)	(e->attr = (o&7))
#define OAM_SET_ATTR_CGB_BNK(e,b)	(e->attr = (b&1) << 3)

struct oam_entry {
	u8 y;		// Y Position
	u8 x;		// X Position
	u8 tile;	// Tile Number
	u8 attr;	// Attributes / Flags
};
