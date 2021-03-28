#include "window.hh"

using namespace std;

int Window::window_count = 0;


Window::Window(int width, int height, int scale) {
	
	window_count++;
	
	if (window_count == 1)
		SDL_Init(SDL_INIT_VIDEO);
	
	window =
		SDL_CreateWindow (
			"Test Screen",				// Title
			SDL_WINDOWPOS_CENTERED,		// X Position
			SDL_WINDOWPOS_CENTERED,		// Y Position
			width * scale,				// Width
			height * scale,				// Height
			0);
	
	renderer =
		SDL_CreateRenderer (
			window, -1,
			SDL_RENDERER_ACCELERATED);
	
	SDL_SetHint(SDL_HINT_RENDER_SCALE_QUALITY, 0);
	SDL_RenderSetLogicalSize(renderer, width, height);
	SDL_RenderSetScale(renderer, scale, scale);
	
	dimensions = { 0, 0, width, height };
	
	//load_chr_rom("./characters.rom");
}


Window::~Window() {
	
	SDL_DestroyTexture(character_rom);
	
	SDL_DestroyRenderer(renderer);
	SDL_DestroyWindow(window);
	
	if (window_count == 1)
		SDL_QuitSubSystem(SDL_INIT_VIDEO);
	
	window_count--;
}


void Window::load_chr_rom(const string &path) {
	
	unique_ptr<char[]> rom_data;
	
	ifstream rom_file(path, ios::binary | ios::ate);
	int length = static_cast<int>(rom_file.tellg());
	
	rom_data = unique_ptr<char[]>(new char[length]);
	
	rom_file.seekg(0, ios::beg);
	rom_file.read(rom_data.get(), length);
	
	rom_file.close();
	
	
	character_rom =
		SDL_CreateTexture (
			renderer,					// Renderer
			SDL_PIXELFORMAT_RGBA8888,	// Format
			SDL_TEXTUREACCESS_TARGET,	// Target
			length >> 3,				// Width
			8);							// Height
	
	
	SDL_SetTextureBlendMode(character_rom, SDL_BLENDMODE_BLEND);
	SDL_SetRenderTarget(renderer, character_rom);
	SDL_SetRenderDrawColor(renderer, 0, 0, 0, 0);
	SDL_RenderClear(renderer);
	SDL_SetRenderDrawColor(renderer, 255, 255, 255, 255);
	
	
	for (int chr = 0; chr < (length >> 6); chr++) {
		
		int chr_start = chr << 6;
		int offset = chr << 3;
		
		for (int y = 0; y < 8; y++) {
			
			int row_start = chr_start + (y << 3);
			
			for (int x = 0; x < 8; x++)
				if (rom_data[row_start + x])
					SDL_RenderDrawPoint(renderer, offset + x, y);
		}
	}
	
	SDL_SetRenderTarget(renderer, NULL);
}


void Window::copy(SDL_Texture *src, SDL_Texture *dst, SDL_Rect *src_rect, SDL_Rect *dst_rect) {

	SDL_SetRenderTarget(renderer, dst);
	SDL_RenderCopy(renderer, src, src_rect, dst_rect);
	SDL_SetRenderTarget(renderer, NULL);
}


void Window::set_colour(const SDL_Colour &c) {
	
	SDL_SetRenderDrawColor(renderer, c.r, c.g, c.b, 0);
	SDL_SetTextureColorMod(character_rom, c.r, c.g, c.b);
}


void Window::reset_colour() {
	
	set_colour({255, 255, 255});
}


void Window::draw_point(Texture &t, int x, int y) {

	SDL_SetRenderTarget(renderer, t.texture);
	SDL_RenderDrawPoint(renderer, x, y);
	SDL_SetRenderTarget(renderer, NULL);
}


void Window::putc(Texture &t, int x, int y, char c) {
	
	SDL_Rect src_rect = { (c - ' ') << 3, 0, 8, 8 };
	SDL_Rect dst_rect = { x, y, 8, 8 };
	
	copy(character_rom, t.texture, &src_rect, &dst_rect);
}


void Window::clear() {
	
	SDL_RenderClear(renderer);
}


void Window::clear(Texture &t) {
	
	SDL_SetRenderTarget(renderer, t.texture);
	clear();
	SDL_SetRenderTarget(renderer, NULL);
}


void Window::clear(Texture &t, SDL_Rect d) {
	
	SDL_SetRenderTarget(renderer, t.texture);
	SDL_RenderDrawRect(renderer, &d);
	SDL_SetRenderTarget(renderer, NULL);
}


void Window::render(Texture &t) {
	
	SDL_Rect dst_rect = t.dimensions;
	dst_rect.h *= t.scale;
	dst_rect.w *= t.scale;
	
	copy(t.texture, NULL, NULL, &dst_rect);
}


void Window::render(Texture &t, SDL_Rect src_rect) {
	
	SDL_Rect dst_rect = t.dimensions;
	dst_rect.h *= t.scale;
	dst_rect.w *= t.scale;
	
	copy(t.texture, NULL, &src_rect, &dst_rect);
}


void Window::render(StreamTexture &t) {
	
	SDL_Rect dst_rect = t.dimensions;
	dst_rect.h *= t.scale;
	dst_rect.w *= t.scale;
	
	copy(t.texture, NULL, NULL, &dst_rect);
}


void Window::display() {
	
	SDL_RenderPresent(renderer);
}


bool Window::get_key_event(SDL_Event &event) {

	while (SDL_WaitEventTimeout(&event, 0)) {
		if (event.type != SDL_KEYUP && event.type != SDL_KEYDOWN)
			continue;
		printf("Keypress\n");
		return true;
	}

	return false;
}


Texture Window::create_texture(int x, int y, int width, int height, int scale) {
	
	SDL_Texture *texture = 
		SDL_CreateTexture (
			renderer,					// Renderer
			SDL_PIXELFORMAT_RGBA8888,	// Format
			SDL_TEXTUREACCESS_TARGET,	// Target
			width,						// Width
			height);					// Height
	
	textures.push_back(texture);
	
	return Texture(*this, texture, { x, y, width, height }, scale);
}


StreamTexture Window::create_stream_texture(int x, int y, int width, int height, int scale) {
	
	SDL_Texture *texture = 
		SDL_CreateTexture (
			renderer,						// Renderer
			SDL_PIXELFORMAT_BGR555,			// Format
			SDL_TEXTUREACCESS_STREAMING,	// Target
			width,							// Width
			height);						// Height
	
	textures.push_back(texture);
	
	return StreamTexture(*this, texture, { x, y, width, height }, scale);
}
