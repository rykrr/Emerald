#pragma once

#include <SDL2/SDL.h>
#include <iostream>
#include <iomanip>
#include <csignal>
#include <unistd.h>

#include <memory>
#include <vector>
#include <string>
#include <sstream>
#include <functional>
#include <map>

#include "keypress.hh"
#include "scroller.hh"

#include "../core/registers.hh"
#include "../memory.hh"
#include "../clock.hh"


typedef std::function<void(u16, u16, u16, u16)> DebugFunction;


struct Command {
	DebugFunction fn;
	u8 argc;
};


class Debugger : public ClockSubscriber {
	
	Window window;
	
	Scroller console_output;
	Texture console_input;
	
	Scroller tracer;
	Texture info;
	
	KeyGenerator keygen;
	
	std::map<std::string, Command> commands;
	
	std::string buffer;
	
	std::vector<u16> breakpoints;
	
	std::vector<std::string> log;
	
	bool active;
	
	bool info_enable;
	bool trace_enable;
	bool log_enable;
	
	bool counter_enable;
	int counter;
	
private:
	
	void print_info();
	void print_trace();
	void render();
	
public:
	
	void operator+=(u8 c);
	void execute(std::string cmd);
	
	void keyup(SDL_Scancode);
	
public:
	
	Debugger();
};
