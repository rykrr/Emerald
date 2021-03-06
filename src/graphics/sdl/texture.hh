#pragma once

#include "window.hh"

#include <SDL2/SDL.h>
#include <iostream>
#include <fstream>
#include <memory>
#include <string>
#include <stdint.h>

typedef SDL_Color SDL_Colour;

class Window;

using namespace std;

class Texture {
	
	friend Window;
	
	Window &parent;
	SDL_Texture	*texture;
	
	SDL_Colour fg_colour;
	SDL_Colour bg_colour;
	
	SDL_Rect dimensions;
	SDL_Rect cursor;
	u16 scale;
	
public:
	
	void set_colour(SDL_Colour fg, SDL_Colour bg) {
		
		fg_colour = fg;
		bg_colour = bg;
	};
	
	void set_fg_colour(SDL_Colour c) { fg_colour = c; };
	void set_bg_colour(SDL_Colour c) { bg_colour = c; };
	
	void set_cursor(int x, int y) {
	
		cursor.x = x % cursor.w;
		cursor.y = y % cursor.h;
	}
	
	void set_position(int x, int y) {
		
		dimensions.x = x;
		dimensions.y = y;
	}
	
	void draw_point(int x, int y, const SDL_Colour &c);
	void draw_point(int x, int y);
	
	void putc(char chr);
	void putb(u8 b);
	void puts(string str);
	void clear(int x, int y, int w, int h);
	void clear();
	void render();
	void render(int x, int y, int w, int h);
	
	void display();
	
private:
	
	Texture(Window &window, SDL_Texture *texture, SDL_Rect dimensions, u16 scale)
		:	parent(window),
			texture(texture),
			dimensions(dimensions),
			cursor {0, 0, dimensions.w >> 3, dimensions.h >> 3},
			scale(scale) {};
		
public:
	
	~Texture() {
		
		SDL_DestroyTexture(texture);
	}
};
