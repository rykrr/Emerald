#include "sprite_fifo.hh"

static void print_entry(struct oam_entry &e) {
	dprintf("{.y = %2d,", e.y);
	dprintf("\t.x = %2d,", e.x);
	dprintf("\t.tile = %2d,", e.tile);
	dprintf("\t.attr = %2X }", e.attr);
	dprintf("\n");
}

bool SpriteFIFO::run(FetchState state) {

	static u16 tile_addr = 0;

	if (!(LCDC & LCDC_SPRITE_ENABLE)) return false;
	
	if (!size || index == size || column >= WIDTH)
		return false;

	switch (state) {
		case FetchState::TILENO:
			
			for (u8 i = 0; i < 8; i++) {
				if (column < entries[index].x) {
					column++;
					break;
				}
				
				if (i == 7)
					return false;
			}
			
			if (entries[index].x < 8)
				discard = 8 - entries[index].x;
			
			tile_addr = TILE_DATA
					+ (entries[index].tile << 4)
					+ ((LY - (entries[index].y - 16)) << 1);
			
			break;
			
			
		case FetchState::DATALO:
			
			tile_data_lo = memory[tile_addr].read_byte();
			break;
			
			
		case FetchState::DATAHI:
			
			tile_data_hi = memory[tile_addr + 1].read_byte();
			break;
			
			
		case FetchState::PUSH:
			
			if (PixelFIFO::size() > 8) return false;
			
			for (u8 mask = 0x80; mask; mask >>= 1) {
				if (discard) {
					discard--;
					continue;
				}
				
				u8 data = (!!(tile_data_hi & mask) << 1)
						| !!(tile_data_lo & mask)
						| (entries[index].attr & OAM_ATTR_PRI)
						| (index << 4);
				
				push(data);
			}
			
			index++;
			column += 8 - discard;
			
			break;
	}

	return true;
}


void SpriteFIFO::scan() {

	if (scan_index >= 1) return;

	if (!(LCDC & LCDC_SPRITE_ENABLE)) return;
	if (scan_index >= 40) return;
	if (size >= 10) return;

	u8 index = scan_index++ << 2;
	
	struct oam_entry entry = {
		.y		= memory[OAM_TABLE + index].read_byte(),
		.x		= memory[OAM_TABLE + index + 1].read_byte(),
		.tile	= memory[OAM_TABLE + index + 2].read_byte(),
		.attr	= memory[OAM_TABLE + index + 3].read_byte(),
	};

	/*
	if (!entry.y || !entry.x) return;
	if (entry.y >= 160) { puts("Rejected for being over 160 y"); return; }
	if (LY < entry.y) { printf("Rejected for being above (LY%d < Y%d)\n", LY, entry.y); return; }
	if (entry.x >= 168) { puts("Rejected for being over 168 x"); return; }
	*/
	
	if (!entry.y || entry.y >= 160) return;
	if (!entry.x || entry.x >= 168) return;
	if (entry.y < 16 && LY >= 16 - entry.y) return;

	if (LY < entry.y - 16) return;
	if (LY >= (entry.y - 16) + (LCDC & LCDC_SPRITE_HEIGHT? 16 : 8))
		return;
	
	//printf("Entry %d accepted\n", scan_index - 1);
	//printf("[LY %02d] [EY %02d]\n", LY, entry.y);
	if (size && entries[size-1].x < entry.x) {
		entries[size++] = entry;
		return;
	}
	
	for (u8 i = 0; i < size; i++) {
		if (entries[i].x < entry.x)
			continue;
		
		dprintf("Swapping entries\n");
		struct oam_entry tmp = entries[i];
		print_entry(tmp);
		print_entry(entry);
		dprintf("--\n");
		entries[i] = entry;
		entry = tmp;
	}
	
	//puts("hi");
	entries[size++] = entry;
}


bool SpriteFIFO::has_priority() {
	return next_priority;
}


u8 SpriteFIFO::get_palette() {
	return next_palette;
}


bool SpriteFIFO::has_pixels(u8 x) {

	if (!size) return false;

	u8 pixel = PixelFIFO::top();
	u8 index = (pixel >> 4) & 7;
	u8 pos = entries[index].x;

	next_priority = !(pixel & OAM_ATTR_PRI);
	next_palette = !!(entries[index].attr & OAM_ATTR_DMG_OBP);
	
	/*
	printf("[Entry %d] [Pos %d] [Want %d] [Prio %d] [Val %d]\n",
			(pixel>>4) & 7, x, pos, next_priority, pixel & 0x0F);
	*/
	
	return (pos - 8 <= x && x < pos);
}


u8 SpriteFIFO::pop() {

	return PixelFIFO::pop() & 0x0F;
}


void SpriteFIFO::reset(u8 column, bool win_enabled) {
	PixelFIFO::reset();
	this->scan_index = 0;
	this->discard = 0;
	this->column = 0;
	this->index = 0;
	this->size = 0;
}
