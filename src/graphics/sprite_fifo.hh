#pragma once
#include "pixel_fifo.hh"
#include "sprite.hh"

#define LCDC_SPRITE_ENABLE	0x02
#define LCDC_SPRITE_HEIGHT	0x04

#define OAM_TABLE 0xFE00
#define TILE_DATA 0x8000

#ifndef DIMENSIONS
#define DIMENSIONS
#define WIDTH 160
#define HEIGHT 144
#endif


class SpriteFIFO : public PixelFIFO {
	
	// Entries that are active on this scanline
	struct oam_entry entries[10];
	u8 index;
	u8 size;

	// OAM Entry To Check
	u8 scan_index;

	// Tile Data
	u8 tile_data_lo;
	u8 tile_data_hi;

	// Vertical Line Number
	u8 column;

	// Discard Pixels at Left Border
	u8 discard;

	// Priority of Next Pixel
	bool next_priority;

	// Palette of Next Pixel
	u8 next_palette;

private:
	
	bool run(FetchState state);
	
public:

	void scan();
	bool has_priority();
	bool has_pixels(u8 x);
	u8 get_palette();
	u8 pop();
	void reset(u8 column = 0, bool win_enabled = false);
	
public:
	
	SpriteFIFO(u8 &LCDC, u8 &LY, u8 &SCX, u8 &SCY)
			: PixelFIFO(LCDC, LY, SCX, SCY) {
		
		reset();
	}
	
};
