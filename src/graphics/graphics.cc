#include "graphics.hh"
#include "../interrupt.hh"

#include "../macros.hh"

#define LINE_CYCLES 114
#define HBLANK_CYCLES 51
#define OAM_CYCLES 20
#define XFR_CYCLES 43


#define BREAK_IF_CLK_LT(c)	\
	if (clock < c) break;	\
	clock -= c;


void Graphics::reset_palettes() {
	
	for (u8 i = 0, m = 3; i < 4; i++, m <<= 2)
		background_palette[i] = BGP & m;
	
	for (u8 i = 0, m = 3; i < 4; i++, m <<= 2)
		sprite_palette[0][i] = OBP0 & m;
	
	for (u8 i = 0, m = 3; i < 4; i++, m <<= 2)
		sprite_palette[1][i] = OBP1 & m;
}


void Graphics::set_mode(VMode mode) {
	
	u8 m = static_cast<u8>(mode);
	STAT = (STAT & ~3) | m;	
	
	if (STAT & 1 << (m + 3))
		interrupt(Interrupt::LCDSTAT);
}


void Graphics::operator+=(u8 cycles) {
	
	static bool win_enabled = false;
	static bool win_active = false;
	
	
	if (!(LCDC & 0x80)) {
		dprintf("LCDC not active %02X\n", LCDC);
		return;
	}
	
	clock += cycles;

	repeat (cycles) {
		if (dma.active) {
			memory.copy(OAM_TABLE + dma.counter, dma.source + dma.counter);
			dma.active = ++dma.counter % 0xA0;
		}
	}
	
	switch (STAT & 3) {
		
		// OAM
		case 2:
			
			repeat (cycles << 1) spfifo.scan();
			BREAK_IF_CLK_LT(OAM_CYCLES);
			dprintf("OAM END\n");
			set_mode(VMode::DRAW);
			break;
		
		
		// DRAW
		case 3:
			
			repeat (cycles << 1) {
				
				bgfifo++;
				spfifo++;
				
				repeat (2) {
					
					if (win_enabled) {
						if (cursor.x <= WX - 7 && cursor.x <= WX + 160 && !win_active)
							bgfifo.reset(cursor.x, win_active = true);
						else if (win_active)
							bgfifo.reset(cursor.x, win_active = false);
					}
					
					if (cursor.x >= 160 || !bgfifo.has_pixels()) break;
					
					u8 pixel = bgfifo.pop();
					
					if ((LCDC & LCDC_SPRITE_ENABLE) && spfifo.has_pixels(cursor.x)) {
						u8 sppixel = spfifo.pop();
						if (sppixel && (!pixel || spfifo.has_priority()))
							pixel = sprite_palette[spfifo.get_palette()][sppixel];
					}
					
					texture << colour_palette[pixel];
					cursor.x++;
				}
			}
			
			if (cursor.x < 160) break;
			
			BREAK_IF_CLK_LT(XFR_CYCLES);
			set_mode(VMode::HBLANK);
			break;
		
		
		// HBLANK
		case 0:
			
			BREAK_IF_CLK_LT(HBLANK_CYCLES);
			
			STAT &= 0xFC;
			
			if (STAT & 0x20 && LY == LYC) {
				STAT |= 4;
				interrupt(Interrupt::LCDSTAT);
			}
			
			if (LY >= 144) {
				set_mode(VMode::VBLANK);
				interrupt(Interrupt::VBLANK);
				break;
			}
			
			win_enabled = (LCDC & LCDC_WINDOW_ENABLE) && (LY <= WY && LY <= WY + 144);
			win_active = false;
				
			bgfifo.reset();
			spfifo.reset();
			cursor.x = 0;
			LY++;
			dprintf("HBLANK END\n");
			reset_palettes();
			set_mode(VMode::OAM);
			repeat (clock << 1) spfifo.scan();
			break;
		
		
		// VBLANK
		case 1:
			
			BREAK_IF_CLK_LT(LINE_CYCLES);
			
			dprintf("VBLANK LY %02X\n", LY);
			
			if (++LY < 154)
				break;
			
			dprintf("VBLANK END\n");
			
			texture.render();
			
			LY = 0;
			set_mode(VMode::OAM);
			break;
	}
}
