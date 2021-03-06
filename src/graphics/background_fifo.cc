#include "background_fifo.hh"

bool BackgroundFIFO::run(FetchState state) {
	
	switch (state) {
		case FetchState::TILENO:
			
			map_base =
				LCDC & (win_enabled	? LCDC_WINDOW_TILE_MAP_SELECT 
									: LCDC_TILE_MAP_SELECT)
				? TILE_MAP_HI : TILE_MAP_LO;
			
			offset.x = (column + SCX) % 0xFF;
			offset.y = (LY + SCY) % 0xFF;
			
			if (!column)
				discard = SCX;
			
			// Note: (offset.y & 0xF8) << 2 == (offset.y >> 3) << 5
			map_index = ((offset.y >> 3) << 5) + (offset.x >> 3);
			tile_no = memory[map_base + map_index].read_byte();
			break;
			
			
		case FetchState::DATALO:
			
			tile_base = TILE_DATA_BLK0;
			
			if (!(LCDC & LCDC_TILE_DATA_SELECT)) {
				tile_base = TILE_DATA_BLK1;
				tile_no = static_cast<i8>(tile_no);
			}
			
			tile_no = (tile_no << 4) + ((offset.y % 8) << 1);
			tile_data_lo = memory[tile_base + tile_no].read_byte();
			break;
			
			
		case FetchState::DATAHI:
			
			tile_data_hi = memory[tile_base + tile_no + 1].read_byte();
			break;
			
			
		case FetchState::PUSH:
			
			if (PixelFIFO::size() > 8) return false;
			
			for (u8 mask = 0x80; mask; mask >>= 1) {
				if (discard) {
					discard--;
					continue;
				}
				
				u8 data = (!!(tile_data_hi & mask) << 1)
						| !!(tile_data_lo & mask);
				
				push(data);
			}
			
			column += 8;
			map_index++;
			break;
	}
	
	return true;
}

void BackgroundFIFO::reset(u8 column, bool win_enabled) {
	PixelFIFO::reset();
	this->win_enabled = win_enabled;
	this->column = column;
}
