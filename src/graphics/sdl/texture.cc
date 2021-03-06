#include "texture.hh"


void Texture::draw_point(int x, int y, const SDL_Colour &c) {

	parent.set_colour(c);
	parent.draw_point(*this, x, y);
}

void Texture::draw_point(int x, int y) {

	parent.set_colour(fg_colour);
	parent.draw_point(*this, x, y);
}

void Texture::putc(char chr) {
	
	parent.set_colour(fg_colour);
	parent.putc(*this, cursor.x << 3, cursor.y << 3, chr);
	
	if (++cursor.x >= cursor.w)
		cursor.y++;
	
	set_cursor(cursor.x, cursor.y);
}

void Texture::putb(u8 b) {
	
	int x = b >> 4;
	putc( x + (x < 10 ? '0' : '7' ));
	
	x = b & 15;
	putc( x + (x < 10 ? '0' : '7' ));
	
}

void Texture::puts(string str) {
	
	for (int i = 0; i < str.length(); i++)
		putc(str[i]);
}

void Texture::clear() {
	
	cursor.x = 0;
	cursor.y = 0;
	parent.set_colour(bg_colour);
	parent.clear(*this);
}

void Texture::clear(int x, int y, int w, int h) {
	
	parent.set_colour(bg_colour);
	parent.clear(*this, {x, y, w, h});
}

void Texture::render() {
	
	parent.render(*this);
}

void Texture::render(int x, int y, int w, int h) {
	
	SDL_Rect tmp = dimensions;
	dimensions.w = w;
	dimensions.h = h;
	parent.render(*this, {x, y, w, h});
	dimensions = tmp;
}

void Texture::display() {
	parent.display();
}
