#pragma once
#include "../memory.hh"
#include "../macros.hh"

#include <cstdio>

extern bool debug_enable;

enum class FetchState : u8 {
	TILENO, DATALO, DATAHI, PUSH
};


class PixelFIFO {
	
	FetchState state;
	
	// Circular Buffer
	u8 pixels[16];
	u8 fifo_size;
	u8 fifo_pos;

protected:

	u8 &LCDC;
	u8 &LY;
	
	u8 &SCY, &SCX;

private:
	
	virtual bool run(FetchState) = 0;
	
public:
	
	void clear() {
		
		state = FetchState::TILENO;
		fifo_size = 0;
		fifo_pos = 0;
	}
	
	
	void reset() {
		
		clear();
		state = FetchState::TILENO;
	}
	
	
	u8 top() {
		return pixels[fifo_pos];
	}
	
	
	u8 pop() {
		
		u8 pixel = pixels[fifo_pos];
		
		// if (fifo_size <= 8) return pixel; // TODO: Change to error
		if (!fifo_size) throw std::runtime_error("Pixel FIFO is empty");
		fifo_size--;
		fifo_pos = ++fifo_pos % 16;
		
		return pixel;
	}
	
	
	void push(u8 pixel) {
		
		if (fifo_size == 16) return;
		pixels[(fifo_pos + fifo_size++) % 16] = pixel;
	}
	
	
	u8 size() { return fifo_size; }
	
	bool has_pixels() { return fifo_size > 8; }
	
	
	void operator++(int i) {
		
		if (!run(state)) return;
		
		switch (state) {
			
			case FetchState::TILENO:
				state = FetchState::DATALO;
				break;

			case FetchState::DATALO:
				state = FetchState::DATAHI;
				break;

			case FetchState::DATAHI:
				state = FetchState::PUSH;
				break;

			case FetchState::PUSH:
				state = FetchState::TILENO;
				break;
		}
	}
	
public:

	PixelFIFO(u8 &LCDC, u8 &LY, u8 &SCX, u8 &SCY)
			:	LCDC(LCDC), LY(LY), SCX(SCX), SCY(SCY) {
		
		reset();
	}
	
};
