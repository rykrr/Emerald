use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::time::{Duration, Instant};

use crate::bus::*;

const CYCLE_DURATION: Duration = Duration::from_nanos(239); // 4194304 Hz ~ 238.663 ns / cycle

pub trait ClockListener {
    fn callback(&mut self, bus: &mut Bus, cycles: u8);
}

type ClockListenerCell = RefCell<dyn ClockListener>;

pub struct Clock {
    callbacks: Vec<Weak<ClockListenerCell>>,
    cycles: u16,
    start_instant: Instant,
    // sleeper: SpinSleeper,
}

impl Clock {
    pub fn new() -> Self {
        Clock {
            callbacks: Vec::new(),
            cycles: 0,
            start_instant: Instant::now(),
            // sleeper: SpinSleeper::default(),
        }
    }

    pub fn attach(&mut self, listener: Rc<ClockListenerCell>) {
        self.callbacks.push(Rc::downgrade(&listener));
    }

    #[inline(always)]
    pub fn increment(&mut self, bus: &mut Bus, cycles: u8) {
        //self.cycles += cycles as u64;

        for listener in &mut self.callbacks {
            listener
                .upgrade()
                .unwrap()
                .borrow_mut()
                .callback(bus, cycles);
        }
    }

    #[inline(always)]
    pub fn cycle_start(&mut self) {
        self.cycles = 0;
        self.start_instant = Instant::now();
    }

    #[inline(always)]
    pub fn cycle_end(&mut self) {
        let elapsed: Duration = self.start_instant.elapsed();
        println!("Elapsed: {}ns", elapsed.as_nanos());

        let expected: Duration = CYCLE_DURATION.saturating_mul(self.cycles as u32);
        println!(
            "{} cycles is approximately {} ns",
            self.cycles,
            expected.as_nanos()
        );

        let remainder: Duration = expected.saturating_sub(elapsed);
        println!("Remainder: {}", remainder.as_nanos());

        if remainder.is_zero() {
            //panic!("Cycle overdue!");
        }
    }
}

use std::fmt;
impl fmt::Debug for Clock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error")
    }
}
