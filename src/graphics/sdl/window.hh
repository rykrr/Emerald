#pragma once

#include "stream_texture.hh"
#include "texture.hh"

#include <SDL2/SDL.h>
#include <iostream>
#include <fstream>
#include <memory>
#include <vector>
#include <string>
#include <stdint.h>

using namespace std;

typedef SDL_Color SDL_Colour;

class StreamTexture;
class Texture;

class Window {
	
	static int window_count;
	
	SDL_Window *window;
	SDL_Renderer *renderer;
	SDL_Texture *character_rom;
	
	vector<SDL_Texture*> textures;
	
	SDL_Rect dimensions;
	
private:
	
	void load_chr_rom(const string &path);
	
	void copy(SDL_Texture *src, SDL_Texture *dst, SDL_Rect *src_rect, SDL_Rect *dst_rect);
	
public:
	
	void draw_point(Texture &t, int x, int y);
	void putc(Texture &t, int x, int y, char c);
	
	void set_colour(const SDL_Colour &c);
	void reset_colour();
	
	void render(StreamTexture &t);
	void render(Texture &t);
	void render(Texture &t, SDL_Rect s);
	
	void clear(Texture &t);
	void clear(Texture &t, SDL_Rect d);
	
	void display();
	void clear();

	bool get_key_event(SDL_Event &e);
	
	Texture create_texture(int x, int y, int width, int height, int scale=1);
	StreamTexture create_stream_texture(int x, int y, int width, int height, int scale=1);
	
public:
	
	Window(int width, int height, int scale=1);
	~Window();
	
};
