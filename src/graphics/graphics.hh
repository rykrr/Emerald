#pragma once

#ifdef THREADED
#include <condition_variable>
#include <thread>
#include <mutex>
#endif

#include "background_fifo.hh"
#include "sprite_fifo.hh"
#include "sdl/stream_texture.hh"
#include "../memory.hh"
#include "../clock.hh"
#include "../macros.hh"

enum class VMode : u8 {
	HBLANK, VBLANK, OAM, DRAW
};


class Graphics : public ClockSubscriber {
	
	Window win;
	StreamTexture texture;
	
	u16 clock;
	struct { u8 x, y; } cursor;
	
	struct {
		u8 active;
		u8 counter;
		u16 source;
	} dma;
	
	BackgroundFIFO bgfifo;
	SpriteFIFO spfifo;

	u16 colour_palette[4];
	u8 background_palette[4];
	u8 sprite_palette[2][4];
	
	u8 LCDC;
	u8 STAT;
	
	u8 LY, LYC;
	
	u8 BGP;
	u8 OBP0, OBP1; // DMG Object Palette

	u8 SCX, SCY;
	u8 WY, WX;
	
	u8 BCPS; // BG Palette Index
	u8 BCPD; // Background Palette Data
	u8 OCPS; // Sprite Palette Index
	u8 OCPD; // Sprite Palette Data
	
	u8 DMA;
	u8 HDMA1, HDMA2, HDMA3, HDMA4, HDMA5;
	
	u8 VBK;
	
private:
	
	void reset_palettes();
	
public:
	
	void set_mode(VMode);
	void operator+=(u8);

	void crash_info() {
#ifdef VDEBUG
		std::cout << std::endl;
		std::cout << "LCDC = " << to_hex(LCDC, 2) << std::endl;
		std::cout << "STAT = " << to_hex(STAT, 2) << std::endl;
		std::cout << "LY   = " << to_hex(LY,   2) << std::endl;
		std::cout << "SCX  = " << to_hex(SCX,  2) << std::endl;
		std::cout << "SCY  = " << to_hex(SCY,  2) << std::endl;
#endif
	}
	
public:
	
	Graphics()
		:	win(160, 144, 3),	texture(win.create_stream_texture(0, 0, 160, 144)),
			clock(0),			cursor { 0, 0 },
			bgfifo(LCDC, LY, SCX, SCY), spfifo(LCDC, LY, SCX, SCY) {
		
		colour_palette[0] = 0x7FFF;
		colour_palette[1] = 0x4210;
		colour_palette[2] = 0x2108;
		colour_palette[3] = 0x0000;
		
		std::vector<std::pair<u8&, u8> > ioregs = {
			{ LCDC,		0x40 }, { STAT,		0x41 },
			{ LY,		0x44 }, { LYC,		0x45 },
			
			{ SCY,		0x42 }, { SCX,		0x43 }, 
			
			{ BGP,		0x47 },
			{ OBP0,		0x48 }, { OBP1,		0x49 },
			
			{ WY,		0x4A }, { WX,		0x4B }, 
			
			{ BCPS,		0x68 }, { BCPD,		0x69 },
			{ OCPS,		0x6A }, { OCPD,		0x6B },
			 
			{ DMA,		0x46 },
			
			{ HDMA1,	0x51 },
			{ HDMA2,	0x52 },
			{ HDMA3,	0x53 },
			{ HDMA4,	0x54 },
			{ HDMA5,	0x55 },
			
			{ VBK,		0x4F }
		};
		
		for (auto &ioreg : ioregs) {
			memory.init_register(ioreg.second, ioreg.first);
			ioreg.first = 0;
		}

		// FF46 DMA
		memory.init_register(0x46, cmem_lambda(addr, byte, write) {
			
			assert(write); assert(byte <= 0xF1);
			dma.counter = 0;
			dma.active = !0;
			dma.source = byte << 8;
			return 0;
		});
	};
};


extern Graphics graphics;
