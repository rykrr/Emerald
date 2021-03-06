#pragma once
#include "../graphics/sdl/texture.hh"
#include "../graphics/sdl/window.hh"
#include "../macros.hh"

#include <string>


class Scroller {
	
	SDL_Rect dimensions;
	
	Texture textures[2];
	bool active;
	u8 line, col;
	
public:
	
	void render() {
		
		textures[!active].set_position(dimensions.x, dimensions.y);
		textures[!active].render(0, line<<3, dimensions.w << 3, (dimensions.h-line)<<3);
		textures[active].set_position(dimensions.x, dimensions.y + ((dimensions.h-line)<<3));
		textures[active].render(0, 0, dimensions.w << 3, line << 3);
	}
	
	void scroll() {
		
		line++;
		col = 0;
		
		render();
		
		if (line >= dimensions.h) {
			active = !active;
			textures[active].clear();
			line = 0;
		}
	}
	
	void clear() {
		
		textures[active].clear();
		textures[active].render();
		
		textures[!active].clear();
		textures[!active].render();
	}
	
	void print(std::string s) {
		
		textures[active].set_cursor(col, line);
		textures[active].puts(s);
		col += s.length();
		render();
	}
	
public:
	
	Scroller(Window &window, int x, int y, int w, int h)
		:	textures {	window.create_texture(x, y, w << 3, h << 3),
						window.create_texture(x, y, w << 3, h << 3)	},
			dimensions({x, y, w, h}),
			active(0), line(0), col(0) {
		
		for_range (i, 2) {
			
			textures[i].set_colour({0, 0, 0, 255}, {255, 255, 255, 255});
			textures[i].clear();
		}
		
		render();
	}
	
};
