#pragma once

#include "window.hh"

#include <iostream>
#include <SDL2/SDL.h>

typedef SDL_Color SDL_Colour;

class Window;

class StreamTexture {
	
	friend Window;
	
	Window &parent;
	SDL_Texture *texture;
	
	SDL_Rect dimensions;
	u16 scale;
	
	u16 *pixels;
	int pitch;
	
	u32 index;
	
public:
	
	void operator<<(u16 pixel);
	void render();
	
public:
	
	StreamTexture(Window &window, SDL_Texture *texture, SDL_Rect dimensions, u8 scale = 1);
	~StreamTexture();
};
