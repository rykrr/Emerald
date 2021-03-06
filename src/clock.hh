#pragma once
#include <functional>
#include <vector>
#include <chrono>

#ifndef clock_lambda
#define clock_lambda(x) [](u8 c) { x += c; }
#endif

struct ClockSubscriber {
	virtual void operator+=(u8) = 0;
};


class Clock {

	// TODO: Get rid of the redundant hooks/subscribers approach
	std::vector<std::function<void(u8)> > hooks;
	std::vector<ClockSubscriber*> subscribers;

	std::chrono::time_point<std::chrono::high_resolution_clock> start_time, end_time;
	u8 cycles;
	
public:
	
	void add_hook(std::function<void(u8)> hook) { hooks.push_back(hook); }
	void add(ClockSubscriber &s) { subscribers.push_back(&s); }

	inline void operator+=(u8 cycles) {

		this->cycles += cycles;

		for (ClockSubscriber *s : subscribers)
			*s += cycles;
		
		for (auto hook : hooks)
			hook(cycles);
	}

	inline void cycle_start() {

		start_time = std::chrono::high_resolution_clock::now();
		cycles = 0;
	}

	inline void cycle_end() {

#ifndef CLOCK_LIMITER_DISABLE
		static const std::chrono::nanoseconds CYCLE_DURATION(240); // 240.385
		end_time = start_time + CYCLE_DURATION * cycles;

		while (std::chrono::high_resolution_clock::now() < end_time) {
			__asm__ volatile("nop");
		}
#endif
	}
};

extern Clock clk;
