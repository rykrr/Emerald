#pragma once

#include <ncurses.h>
#include <signal.h>
#include <iomanip>
#include <string>
#include <vector>
#include <deque>
#include <map>

#include "../core/registers.hh"
#include "../memory.hh"
#include "../clock.hh"

#define debug_lambda [&](u16 a, u16 b, u16 c, u16 d)

#define print_byte(w, y, x, data) {				\
	stream << to_hex(data, 2);					\
	mvwprintw(w, y, x, stream.str().c_str());	\
	stream.str("");								\
	stream.clear();								\
}

#define print_word(w, y, x, data) {				\
	stream << to_hex(data, 4);					\
	mvwprintw(w, y, x, stream.str().c_str());	\
	stream.str("");								\
	stream.clear();								\
}

#define print_bits(w, y, x, data) {				\
	wmove(w, y, x);								\
	for (u8 i = 0x80; i; i >>= 1)				\
		wprintw(w, !!(data & i)? "1" : "0");	\
}

#define TRACE_WIDTH 21
#define CONSOLE_WIDTH 72
#define CONSOLE_HEIGHT (height - 7)


void winch_handler(int);

typedef std::function<void(u16, u16, u16, u16)> DebugFunction;


struct DebugCommand {
	
	DebugFunction fn;
	u8 argc;
};


class Debugger : public ClockSubscriber {

	std::vector<u16> breakpoints;
	std::map<std::string, DebugCommand> commands;

	std::deque<std::string> console_output;
	std::deque<std::string> trace;

	WINDOW *cpu_win;
	WINDOW *ppu_win;
	WINDOW *int_win;
	WINDOW *ext_win;

	WINDOW *console_win;
	WINDOW *trace_win;
	WINDOW *mode_win;

	int width;
	int height;

	bool debug_active;
	bool fast_mode;

	bool logging;
	std::string log;

	int counter;

	std::string opstr = "";
	int cb = 0;

private:

	void resolve_opstr() {

		using namespace Registers;

		opstr = instruction_strings[cb][memory[PC].read_byte()];
		
		std::stringstream ip;
		ip << to_hex(PC, 4);
		
		std::stringstream in;
		in << to_hex(memory[PC].read_byte(), 2);
		
		cb = (!cb && memory[PC].read_byte() == 0xCB);
		opstr = ip.str() + " " + in.str() + " " + opstr;
		
		if (opstr.find("xxxx") != opstr.npos) {
			std::stringstream hex;
			hex << to_hex(memory[PC+1].read_word(), 4);
			opstr.replace(opstr.find("xxxx"), 4, hex.str());
		}
		
		if (opstr.find("xx") != opstr.npos) {
			std::stringstream hex;
			hex << to_hex(memory[PC+1].read_byte(), 2);
			opstr.replace(opstr.find("xx"), 2, hex.str());
		}
	}

	void print_trace() {

		trace.push_back(opstr);

		if (trace.size() > (height - 6))
			trace.pop_front();

		for (int i = 0; i < trace.size(); i++) {
			mvwprintw(trace_win, i + 1, 2, trace[i].c_str());
			for (int j = 0; j < TRACE_WIDTH - trace[i].length(); j++)
				wprintw(trace_win, " ");
		}

		wrefresh(trace_win);
	}

	void print_registers() {
		
		using namespace Registers;
		static std::stringstream stream;
		
		print_word(cpu_win, 1, 5,	PC);
		print_word(cpu_win, 2, 5,	SP);
		print_word(cpu_win, 1, 15,	AF);
		print_word(cpu_win, 2, 15,	DE);
		print_word(cpu_win, 1, 25,	BC);
		print_word(cpu_win, 2, 25,	HL);

		wrefresh(cpu_win);

		print_bits(int_win, 1, 5, memory[0xFF0F].read_byte());
		print_bits(int_win, 2, 5, memory[0xFFFF].read_byte());

		wrefresh(int_win);

		print_byte(ppu_win, 1, 7, memory[0xFF40].read_byte());
		print_byte(ppu_win, 2, 7, memory[0xFF41].read_byte());
		print_byte(ppu_win, 1, 16, memory[0xFF42].read_byte());
		print_byte(ppu_win, 2, 16, memory[0xFF43].read_byte());
		print_byte(ppu_win, 1, 25, memory[0xFF45].read_byte());
		print_byte(ppu_win, 2, 25, memory[0xFF44].read_byte());

		wrefresh(ppu_win);

		print_byte(ext_win, 1, 7, memory[0xFF00].read_byte());

		wrefresh(ext_win);
	}

	void draw_static() {

		box(cpu_win, 0, 0);
		mvwprintw(cpu_win, 0, 1,	"CPU");
		mvwprintw(cpu_win, 1, 2,	"PC");
		mvwprintw(cpu_win, 2, 2,	"SP");
		mvwprintw(cpu_win, 1, 12,	"AF");
		mvwprintw(cpu_win, 2, 12,	"DE");
		mvwprintw(cpu_win, 1, 22,	"BC");
		mvwprintw(cpu_win, 2, 22,	"HL");
		wrefresh(cpu_win);

		box(int_win, 0, 0);
		mvwprintw(int_win, 0, 1, "INT");
		mvwprintw(int_win, 1, 2, "IF");
		mvwprintw(int_win, 2, 2, "IE");
		wrefresh(int_win);

		box(ppu_win, 0, 0);
		mvwprintw(ppu_win, 0, 1, "PPU");
		mvwprintw(ppu_win, 1, 2, "LCDC");
		mvwprintw(ppu_win, 2, 2, "STAT");
		mvwprintw(ppu_win, 1, 12, "SCY");
		mvwprintw(ppu_win, 2, 12, "SCX");
		mvwprintw(ppu_win, 1, 21, "LYC");
		mvwprintw(ppu_win, 2, 21, "LY");
		wrefresh(ppu_win);

		box(ext_win, 0, 0);
		mvwprintw(ext_win, 0, 1, "EXT");
		mvwprintw(ext_win, 1, 2, "JOYP");
		wrefresh(ext_win);

		box(trace_win, 0, 0);
		mvwprintw(trace_win, 0, 1, "Trace");

		box(console_win, 0, 0);
		mvwprintw(console_win, 0, 1, "Console");
		wrefresh(console_win);

		box(trace_win, 0, 0);
		mvwprintw(trace_win, 0, 1, "Trace");
		wrefresh(trace_win);
	}

	void create_windows() {

		int midpoint = 99;
		getmaxyx(stdscr, height, width);

		if (width < midpoint)
			throw std::runtime_error("Terminal too small");

		midpoint = (width - midpoint) >> 1;

		cpu_win = newwin(4, 31, 0, midpoint);
		ppu_win = newwin(4, 29, 0, midpoint + 46);
		int_win = newwin(4, 15, 0, midpoint + 31);
		ext_win = newwin(4, 15, 0, midpoint + 75);

		console_win = newwin(height - 4, 99 - 24, 4, midpoint + 24);
		trace_win = newwin(height - 4, 24, 4, midpoint);

		mode_win = newwin(5, 21, (height - 5) >> 1, (width - 21) >> 1);
		box(mode_win, 0, 0);
		mvwprintw(mode_win, 2, 2, "Fast Mode Enabled");

		draw_static();
		print_registers();
	}

	void delete_windows() {

#define del(x) if(x) delwin(x); x = NULL;
		del(cpu_win);
		del(ppu_win);
		del(int_win);
		del(ext_win);
		del(console_win);
		del(trace_win);
		del(mode_win);
#undef del
	}

	void console_scroll(std::string s = "") {

		if (console_output.size() > CONSOLE_HEIGHT)
			console_output.pop_front();

		if (s.length())
			console_output.push_back(s);

		for (int i = 0; i < console_output.size(); i++) {
			mvwprintw(console_win, i + 1, 2, console_output[i].c_str());
			for (int j = 0; j < CONSOLE_WIDTH - console_output[i].length(); j++)
				wprintw(console_win, " ");
		}
	}

	void execute(std::string s) {

		std::stringstream stream(s);
		std::string cmd;
		stream >> cmd;
		
		if (!cmd.length())
			return;
		
		auto cpos = commands.find(cmd);
		
		if (cpos == commands.end()) {
			console_scroll("invalid command.");
			return;
		}
		
		DebugCommand command = commands[cmd];
		u16 args[4];
		
		for_range (i, 4) args[i] = 0;
		
		for_range (i, command.argc) {
			stream >> std::hex >> args[i];
			
			if (!stream) {
				console_scroll("bad argument.");
				return;
			}
		}
	
		stream.str("");
		command.fn(args[0], args[1], args[2], args[3]);
	}

	void console() {

		char input[CONSOLE_WIDTH];

		while (debug_active) {

			console_scroll();
			wmove(console_win, console_output.size() + 1, 1);
			for (int i = 0; i < CONSOLE_WIDTH; i++)
				wprintw(console_win, " ");
			mvwprintw(console_win, console_output.size() + 1, 2, "> ");

			curs_set(1);
			echo();
			wgetnstr(console_win, input, CONSOLE_WIDTH-4);
			console_scroll("> " + std::string(input));
			noecho();
			curs_set(0);

			execute(input);
		}
	}

public:

	void fatal() {

	}

	void refresh() {
		delete_windows();
		create_windows();
	}

	void operator+=(u8 c) {

		if (!fast_mode || logging) {

			resolve_opstr();

			if (!fast_mode) {
				print_registers();
				print_trace();
			}

			if (logging)
				log += opstr + "\n";;
		}

		if (!debug_active) {

			for (u16 b : breakpoints) {
				if (Registers::PC == b) {
					std::stringstream stream;
					stream << "Breakpoint " << to_hex(Registers::PC, 4) << " reached.";
					console_scroll(stream.str());
					debug_active = true;
					counter = -1;
					break;
				}
			}
		}

		if (!debug_active) {

			int ch;
			wtimeout(console_win, 0);
			while ((ch = wgetch(console_win)) != ERR) {
				if (ch == '\t')
					counter = 0;

				if (ch == 'f') {
					fast_mode = !fast_mode;

					if (fast_mode) {
						wclear(trace_win);
						trace.clear();
						wclear(cpu_win);
						wclear(ppu_win);
						wclear(int_win);
						draw_static();
					}
				}
			}
			wtimeout(console_win, -1);
			
			if (0 < counter) counter--;
			if (counter) return;
			debug_active = true;
		}

		console();
	}
	
public:

	Debugger(): debug_active(false), logging(true), counter(1) {

		initscr();
		raw();
		noecho();
		curs_set(0);
		keypad(console_win, true);
		signal(SIGWINCH, winch_handler);
		create_windows();

		commands["peek"] = {
			debug_lambda {
				std::stringstream stream;
				stream << to_hex(memory[a].read_byte(), 2);
				console_scroll(stream.str());
			}, 1
		};
		
		commands["poke"] = {
			debug_lambda {
				memory.set_debug_mode(true);
				memory[a].write_byte(b);
				memory.set_debug_mode(false);
			}, 2
		};
		
		commands["exit"] = {
			debug_lambda {
				memory.set_debug_mode(true);
				memory[Registers::PC].write_byte(0x10);
				memory.set_debug_mode(false);
				debug_active = false;
				cpu.stop();
			}, 0
		};

		commands["nofast"] = {
			debug_lambda {
				fast_mode = false;
			}, 0
		};

		commands["fast"] = {
			debug_lambda {
				fast_mode = true;
			}, 0
		};
		
		commands["jump"] = {
			debug_lambda {
				Registers::PC = a;
			}, 1
		};

		commands["run"] = {
			debug_lambda {
				debug_active = false;
				counter = -1;
			}, 0
		};

		commands["next"] = {
			debug_lambda {
				counter = a;
				debug_active = false;
			}, 1
		};

		commands["s"] = {
			debug_lambda {
				counter = 1;
				debug_active = false;
			}, 0
		};
		
		commands["view"] = {
			debug_lambda {
					
				std::stringstream stream;
				
				for (u16 i = 0; a + (i<<4) < b; i++) {
					
					c = a + (i<<4);
					
					stream << "[" << to_hex(c, 4) << "] ";
					
					for (u8 j = 0; c + j < b && j < 16; j++)
						stream << to_hex(memory[c+j].read_byte(), 2) << " ";

					console_scroll(stream.str());
					stream.str("");
					stream.clear();
				}
			}, 2
		};
		
		commands["brlist"] = {
			debug_lambda {
				
				std::stringstream stream;
				
				for_range (i, breakpoints.size()) {
					
					stream	<< to_hex(i, 2)
							<< ": " << to_hex(breakpoints[i], 4);
					console_scroll(stream.str());
					stream.str("");
					stream.clear();
				}
			}, 0
		};
		
		commands["bradd"] = {
			debug_lambda {
				breakpoints.push_back(a);
			}, 1
		};
		
		commands["brdel"] = {
			debug_lambda {
				if (breakpoints.size() <= a)
					return;
				breakpoints.erase(breakpoints.begin()+a);
			}, 1
		};

		commands["oam"] = {
			debug_lambda {
				
				std::stringstream stream;
				
				for (u8 i = 0; i < 40; i += 4) {
					for (u8 j = 0; j < 4; j++) {
						u8 index = (i + j) << 2;
						stream	<< to_hex(memory[OAM_TABLE + index + 0].read_byte(), 2)	<< " "
								<< to_hex(memory[OAM_TABLE + index + 1].read_byte(), 2)	<< " "
								<< to_hex(memory[OAM_TABLE + index + 2].read_byte(), 2)	<< " "
								<< to_hex(memory[OAM_TABLE + index + 3].read_byte(), 2)	<< "   ";
					}
					console_scroll(stream.str());
					stream.str("");
					stream.clear();
				}
			}, 0
		};

		commands["log"] = {
			debug_lambda {
				log.clear();
				logging = true;
			}, 0
		};

		commands["nolog"] = {
			debug_lambda {
				logging = false;
			}, 0
		};

		commands["dump"] = {
			debug_lambda {
				std::ofstream dump("/tmp/emerald.log");
				dump << log;
			}, 0
		};
		
		commands["clear"] = {
			debug_lambda {
				console_output.clear();
				wclear(console_win);
				draw_static();
			}, 0
		};
	}

	~Debugger() {

		delete_windows();
		endwin();
	}
} debugger;


void winch_handler(int _) {
	signal(SIGWINCH, winch_handler);
	debugger.refresh();
}
