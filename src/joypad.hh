#pragma once

#include "graphics/sdl/window.hh"
#include "memory.hh"
#include "clock.hh"

#define JOYP_BTNS	0x20
#define JOYP_DPAD	0x10

#define JOYP_START	0x08
#define JOYP_SELECT	0x04
#define JOYP_B		0x02
#define JOYP_A		0x01

#define JOYP_DOWN	0x08
#define JOYP_UP		0x04
#define JOYP_LEFT	0x02
#define JOYP_RIGHT	0x01


#define JOYP_SET(KEYUP, BANK, CODE)	\
	if (KEYUP) BANK &= ~CODE;		\
	else BANK |= CODE;				\


class Joypad : public ClockSubscriber {

	Window &window;

	u8 buttons;
	u8 directions;

private:

	void scan(SDL_Event event) {

		bool change = false;
		bool keyup = event.type == SDL_KEYUP;

		#define JOYP_CASE(SCANCODE, BANK, CODE)		\
			case SCANCODE:							\
				JOYP_SET(keyup, BANK, CODE);		\
				change = true;						\
			break;

		#define JOYP_BTN_CASE(SCANCODE, CODE)		\
				JOYP_CASE(SCANCODE, buttons, CODE);

		#define JOYP_DIR_CASE(SCANCODE, CODE)		\
				JOYP_CASE(SCANCODE, directions, CODE);

		switch (event.key.keysym.scancode) {
			// Buttons
			JOYP_BTN_CASE(SDL_SCANCODE_F1, JOYP_SELECT);
			JOYP_BTN_CASE(SDL_SCANCODE_F2, JOYP_START);
			JOYP_BTN_CASE(SDL_SCANCODE_E,  JOYP_A);
			JOYP_BTN_CASE(SDL_SCANCODE_Q,  JOYP_B);

			// Directions
			JOYP_DIR_CASE(SDL_SCANCODE_DOWN,  JOYP_DOWN);
			JOYP_DIR_CASE(SDL_SCANCODE_UP,    JOYP_UP);
			JOYP_DIR_CASE(SDL_SCANCODE_LEFT,  JOYP_LEFT);
			JOYP_DIR_CASE(SDL_SCANCODE_RIGHT, JOYP_RIGHT);
		}

		#undef JOYP_DIR_CASE
		#undef JOYP_BTN_CASE
		#undef JOYP_CASE

		if (!keyup && change)
			interrupt(Interrupt::JOYPAD);
	}

public:

	void operator+=(u8 c) {
		unused(c);

		SDL_Event event;
		while (window.get_key_event(event))
			scan(event);
	}

public:

	Joypad(Window &w) : window(w) {

		memory.init_register(0x00, [this](u16 addr, u8 data, bool write) {

			static u8 prev = 0xFF;

			SDL_Event event;
			while (window.get_key_event(event))
				scan(event);

			data = data & 0xF0;

			if (data == 0x30)
				return (u8) (data | 0x0F);

			if (!data)
				return prev;

			printf("Data %02X\n", data);
			if (!(data & JOYP_BTNS)) {
				data |= (~buttons) & 0x0F;
				printf("BTNS %02X\n", data);
			}

			if (!(data & JOYP_DPAD)) {
				data |= (~directions) & 0x0F;
				printf("DPAD %02X\n", data);
			}

			prev = data;
			return data;
		});
	}
};
