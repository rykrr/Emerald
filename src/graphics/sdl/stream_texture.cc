#include "stream_texture.hh"

#include <cstdio>
	

StreamTexture::StreamTexture(Window &window, SDL_Texture *texture, SDL_Rect dimensions, u8 scale)
	:	parent(window),
		texture(texture),
		dimensions(dimensions),
		scale(scale),
		index(0)		{
	
	SDL_LockTexture(texture, NULL, (void**) &pixels, &pitch);
	pitch >>= 1;
	pitch *= 144;
}
	

StreamTexture::~StreamTexture() {
	
	SDL_UnlockTexture(texture);
}


void StreamTexture::operator<<(u16 pixel) {
	
	if (index >= pitch)
		return; // TODO Error
	
	pixels[index++] = pixel;
}


void StreamTexture::render() {
	
	SDL_UnlockTexture(texture);
	parent.render(*this);
	parent.display();
	SDL_LockTexture(texture, NULL, (void**) &pixels, &pitch);
	
	pitch >>= 1;
	pitch *= 144;
	
	index = 0;
}
