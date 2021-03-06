#pragma once
#include <SDL2/SDL.h>
#include <functional>
#include <vector>


typedef std::function<void(SDL_Scancode)> KeyHandlerFunction;


class KeyGenerator {
	
	std::vector<KeyHandlerFunction> keyup_handlers;
	std::vector<KeyHandlerFunction> keydn_handlers;
	
private:
	
	void handle(SDL_Event event) {
			
			if (event.type != SDL_KEYUP && event.type != SDL_KEYDOWN)
				return;
			
			#define HANDLE(handlers)						\
				for (int i = 0; i < handlers.size(); i++)	\
					handlers[i](event.key.keysym.scancode);	\
				return;
			
			switch (event.type) {
				case SDL_KEYUP:
					HANDLE(keyup_handlers);
					break;
				
				case SDL_KEYDOWN:
					HANDLE(keydn_handlers);
					break;
			}
	}
	
public:
	
	void keyup_subscribe(KeyHandlerFunction fn) {
		keyup_handlers.push_back(fn);
	};
	
	
	void keydn_subscribe(KeyHandlerFunction fn) {
		keydn_handlers.push_back(fn);
	};
	
	void wait() {
		
		SDL_Event event;
		SDL_WaitEvent(&event);
		handle(event);
	}
	
	void poll() {
		
		SDL_Event event;
		
		while (SDL_PollEvent(&event))
			handle(event);
	}
};
