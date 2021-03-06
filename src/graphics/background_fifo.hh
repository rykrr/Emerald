#pragma once
#include "pixel_fifo.hh"

#define LCDC_TILE_MAP_SELECT		0x08
#define LCDC_TILE_DATA_SELECT		0x10
#define LCDC_WINDOW_TILE_MAP_SELECT	0x30

#define LCDC_WINDOW_ENABLE	0x20

#define TILE_MAP_LO 0x9800
#define TILE_MAP_HI 0x9C00
#define TILE_DATA_BLK0 0x8000
#define TILE_DATA_BLK1 0x8800

#ifndef DIMENSIONS
#define DIMENSIONS
#define WIDTH 160
#define HEIGHT 144
#endif


class BackgroundFIFO : public PixelFIFO {
	
	// Map Address Base Pointer
	u16 map_base;
	
	// Current Position in Map
	u16 map_index;
	
	// Tile Number
	i16 tile_no;
	
	// Tile Data Base Pointer
	u16 tile_base;
	
	// Tile Data
	u8 tile_data_lo;
	u8 tile_data_hi;
	
	// Vertical Line Number
	u8 column;

	// Discard First Pixels
	u8 discard;

	u8 win_enabled;
	
	// Pixel Offsets
	struct { u8 x, y; } offset;
		
private:
	
	bool run(FetchState state);
	
public:
	
	void reset(u8 column = 0, bool win_enabled = false);
	
public:
	
	BackgroundFIFO(u8 &LCDC, u8 &LY, u8 &SCX, u8 &SCY)
			: PixelFIFO(LCDC, LY, SCX, SCY) {
		
		reset();
	}
	
};
