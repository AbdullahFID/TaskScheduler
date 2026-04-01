#![allow(dead_code)]
// generator.rs - random task generator for stress testing
// this file is not really part of the core scheduler logic
// its just a helper that creates fake tasks so we can test the system
// without having to manually write csv files every time
// uses a simple LCG (linear congruential generator) for random numbers
// so we dont need any external crates/libraries - everything from scratch

use crate::task::{Task, MAX_TASKS};

// simple linear congruential generator (like the one in glibc)
// this is one of the oldest and simplest ways to make "random" numbers
// the formula is: next = (current * a) + c
// where a and c are carefully chosen constants (not random, from math research)
// its not truly random (its deterministic) but it looks random enough for our purposes
struct Rng {
    state: u64, // the current state, gets transformed each time we call next()
}

impl Rng {
    // create a new rng with a starting seed
    // same seed = same sequence of "random" numbers every time
    // this is actually a feature not a bug - makes our tests reproducible
    fn new(seed: u64) -> Rng {
        Rng { state: seed }
    }

    // generate the next random number
    // the two big constants (6364136223846793005 and 1442695040888963407) are from the PCG paper
    // they were specifically chosen to give a full cycle (hits every possible number before repeating)
    // wrapping_mul and wrapping_add let the numbers overflow on purpose without crashing
    // the overflow IS the randomness - numbers get huge, wrap around, and the leftovers look random
    // >> 33 takes just the top 31 bits which have better statistical properties
    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        return self.state >> 33;
    }

    // random integer in range [lo, hi] inclusive
    // uses modulo to squish the big random number into our desired range
    // example: range(1, 4) gives us 1, 2, 3, or 4
    fn range(&mut self, lo: i32, hi: i32) -> i32 {
        let span = (hi - lo + 1) as u64; // how many possible values (4-1+1 = 4)
        let val = self.next() % span;     // random number between 0 and span-1
        return lo + val as i32;            // shift it into our range
    }
}

// list of task names to randomly pick from
// these are warehouse/robot themed to match our simulation scenario
const NAMES: [&str; 10] = [
    "pick_up_box",
    "scan_shelf",
    "deliver_pkg",
    "charge_batt",
    "sort_items",
    "inspect_area",
    "move_pallet",
    "update_inv",
    "clean_zone",
    "emergency_chk",
];

// generate count tasks and write them into the output array
// arrival_spread controls how spread out the arrival times are:
//   0 = all tasks arrive at time 0 (burst mode, everything hits at once)
//   100 = tasks arrive randomly between time 0 and 100 (normal spread)
// seed is the starting value for the random number generator
// returns how many tasks were actually generated (capped at MAX_TASKS)
pub fn generate_tasks(
    output: &mut [Task; MAX_TASKS],
    count: usize,
    arrival_spread: i32,
    seed: u64,
) -> usize {
    let mut rng = Rng::new(seed);
    let num = if count > MAX_TASKS { MAX_TASKS } else { count }; // dont go over the limit

    let mut i = 0;
    while i < num {
        let mut t = Task::empty(); // start with a blank task
        t.id = (i as i32) + 1;     // ids start at 1, not 0

        // pick a random name from the list
        let name_idx = rng.range(0, 9) as usize; // 0-9 because we have 10 names
        t.set_name(NAMES[name_idx]);

        // random priority 1-4 (LOW to CRITICAL)
        t.priority = rng.range(1, 4);

        // arrival time depends on the spread parameter
        if arrival_spread > 0 {
            t.arrival = rng.range(0, arrival_spread); // random arrival within the spread
        } else {
            t.arrival = 0; // burst mode - everyone arrives at T=0
        }

        // deadline between 10 and 120 seconds after arrival
        // this is the relative deadline (how many seconds after arrival it needs to be done)
        t.deadline = rng.range(10, 120);

        // duration between 1 and 20 seconds (how long the task actually takes to run)
        t.duration = rng.range(1, 20);

        t.order = i as i32; // arrival order for FIFO tiebreaking

        output[i] = t; // put the task in the output array
        i = i + 1;
    }

    return num; // return how many we generated
}
