#![allow(dead_code)]
// generator.rs - random task generator for stress testing
// uses a simple LCG so we dont need any external crates
// irrelevant to the codebase just creates csv

use crate::task::{Task, MAX_TASKS};

// simple linear congruential generator (like the one in glibc)
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Rng {
        Rng { state: seed }
    }

    // get next random number
    fn next(&mut self) -> u64 {
        // standard LCG constants
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        return self.state >> 33;
    }

    // random int in range [lo, hi] inclusive
    fn range(&mut self, lo: i32, hi: i32) -> i32 {
        let span = (hi - lo + 1) as u64;
        let val = self.next() % span;
        return lo + val as i32;
    }
}

// list of task names to pick from
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

// generate count tasks, writing into the output array
// arrival_spread controls how spread out arrivals are
// 0 = all at time 0 (burst mode), higher = more spread
pub fn generate_tasks(
    output: &mut [Task; MAX_TASKS],
    count: usize,
    arrival_spread: i32,
    seed: u64,
) -> usize {
    let mut rng = Rng::new(seed);
    let num = if count > MAX_TASKS { MAX_TASKS } else { count };

    let mut i = 0;
    while i < num {
        let mut t = Task::empty();
        t.id = (i as i32) + 1;

        // pick a random name
        let name_idx = rng.range(0, 9) as usize;
        t.set_name(NAMES[name_idx]);

        // random priority 1-4
        t.priority = rng.range(1, 4);

        // arrival time depends on spread
        if arrival_spread > 0 {
            t.arrival = rng.range(0, arrival_spread);
        } else {
            t.arrival = 0; // burst mode - everyone arrives at once
        }

        // deadline between 10 and 120 seconds after arrival
        t.deadline = rng.range(10, 120);

        // duration between 1 and 20 seconds
        t.duration = rng.range(1, 20);

        t.order = i as i32;

        output[i] = t;
        i = i + 1;
    }

    return num;
}

