#![allow(dead_code)]
// task.rs - task struct and priority stuff
// this is basically the "blueprint" for what a task looks like in our system
// every task in the simulation is one of these structs

// hard cap on how many tasks we can have, 1024 is more than enough for our simulation
// we use this instead of dynamic arrays (Vec) because prof said no external/dynamic stuff
pub const MAX_TASKS: usize = 1024;

// max characters a task name can be, 64 chars should be plenty for names like "pick_up_box"
pub const MAX_NAME: usize = 64;

// priority levels (higher number = more urgent)
// so like if a robot has to pick up a box (LOW=1) vs an emergency stop (CRITICAL=4),
// the emergency stop always goes first no matter what
pub const PRI_LOW: i32 = 1;
pub const PRI_MED: i32 = 2;
pub const PRI_HIGH: i32 = 3;
pub const PRI_CRIT: i32 = 4;

// status codes for tracking where a task is in its lifecycle
// every task goes through these stages in order: PENDING -> QUEUED -> READY -> RUNNING -> DONE
// its like a package being shipped: ordered -> warehouse -> on truck -> delivering -> delivered
pub const ST_PENDING: i32 = 0;   // task exists but hasnt arrived at the scheduler yet
pub const ST_QUEUED: i32 = 1;    // task arrived and is sitting in the arrival queue waiting
pub const ST_READY: i32 = 2;     // task got promoted to the priority heap, eligible to run
pub const ST_RUNNING: i32 = 3;   // task is currently being executed by the cpu
pub const ST_DONE: i32 = 4;      // task finished executing, we're done with it

// #[derive(Clone, Copy)] is Rusts equievelnt to struct Task copy = original; in C
// it lets us copy this struct around by just copying the raw bytes, no pointers no heap allocation
// we NEED this because we store tasks in arrays and swap them around during sorting
#[derive(Clone, Copy)]
pub struct Task {
    pub id: i32,
    pub name: [u8; MAX_NAME], // i would have used strings but the prof said i cant i have to do everything manually
                               // so we store the name as raw bytes in a fixed size array, like a char[] in C
                               // its not pretty but it works and its what they wanted
    pub name_len: usize,       // how many of those 64 bytes are actually part of the name
                               // like if name is "pick_up_box" thats 11 bytes, the other 53 are just zeros
    pub priority: i32,         // 1=LOW, 2=MED, 3=HIGH, 4=CRITICAL
    pub arrival: i32,          // what time (in seconds) does this task show up at the scheduler
    pub deadline: i32,         // seconds AFTER arrival that the task needs to be done by
                               // so if arrival=10 and deadline=30, task must finish by T=40
    pub duration: i32,         // how many seconds this task takes to actually run
    pub status: i32,           // current lifecycle stage (ST_PENDING through ST_DONE)
    pub order: i32,            // what order did this task arrive in, used for fifo tiebreaking
                               // if two tasks have identical priority+deadline+duration, whoever came first wins
}

impl Task {
    // creates a blank task with all fields zeroed out
    // Rust doesnt have null so we cant just say "Task t = null" like in java
    // instead we make an "empty" task where everything is 0 and status is PENDING
    // we use this to fill the task array at the start before loading real data
    pub fn empty() -> Task {
        Task {
            id: 0,
            name: [0u8; MAX_NAME],  // fill all 64 bytes with 0
            name_len: 0,
            priority: 0,
            arrival: 0,
            deadline: 0,
            duration: 0,
            status: ST_PENDING,
            order: 0,
        }
    }

    // set the name from a string slice
    // basically takes a normal string like "pick_up_box" and manually copies it byte by byte
    // into our fixed size byte array. this is equievelnt to strncpy in C
    pub fn set_name(&mut self, s: &str) {
        let bytes = s.as_bytes();  // turn the string into raw bytes
        let len = if bytes.len() > MAX_NAME { MAX_NAME } else { bytes.len() }; // cap it to max_name to prveent overflow
        let mut i = 0;
        while i < len {  // copy each byte one at a time into our array
            self.name[i] = bytes[i];
            i = i + 1;
        }
        self.name_len = len;  // remember how many bytes we actually stored
    }

    // get the name back as a readable string
    // does the reverse of set_name - takes our raw bytes and turns them back into text
    // we only read name[0..name_len] because the rest is just zeros/garbage
    pub fn get_name(&self) -> &str {
        match std::str::from_utf8(&self.name[..self.name_len]) {
            Ok(s) => s,     // bytes are valid text, return the string
            Err(_) => "???", // if data is corrupted somehow just return ??? so we dont crash
        }
    }

    // priority rank for heap ordering (lower number = more urgent)
    // the min-heap puts the SMALLEST number on top, so we flip the priority:
    // CRITICAL(4) becomes rank 0 (goes to top), LOW(1) becomes rank 3 (goes to bottom)
    // basically: 4 - 4 = 0, 4 - 3 = 1, 4 - 2 = 2, 4 - 1 = 3
    pub fn rank(&self) -> i32 {
        return 4 - self.priority;
    }

    // absolute deadline = arrival time + relative deadline
    // the csv stores deadline as "seconds after arrival" but for comparing tasks
    // we need the actual clock time the task must be done by
    // example: task arrives at T=5, deadline is 30 seconds -> must finish by T=35
    pub fn abs_deadline(&self) -> i32 {
        return self.arrival + self.deadline;
    }

    // turns the priority number into a human readable string for printing
    // just so the output says "CRITICAL" instead of "4" which is way easier to read
    pub fn priority_str(&self) -> &str {
        if self.priority == PRI_CRIT { return "CRITICAL"; }
        if self.priority == PRI_HIGH { return "HIGH"; }
        if self.priority == PRI_MED { return "MEDIUM"; }
        return "LOW";
    }
}

// simple bubble sort for task array by arrival time
// O(n^2) def not the "best" but easiest for me, could have done quick sort but too hard
// and also wouldnt make much a difference if <= 1024 tasks (max limit)
// how it works: go through the array over and over, compare neighbors, swap if out of order
// like bubbles rising to the top - the biggest values "bubble" to the end each pass
// after pass 1 the latest arrival is at the end, after pass 2 the second latest, etc
pub fn sort_by_arrival(tasks: &mut [Task], count: usize) {
    let mut i = 0;
    while i < count {
        let mut j = 0;
        while j < count - 1 - i {  // -i because the last i elements are already sorted
            if tasks[j].arrival > tasks[j + 1].arrival {
                // neighbor on the left arrives later than neighbor on the right, swap em
                let tmp = tasks[j];       // save the left one
                tasks[j] = tasks[j + 1];  // overwrite left with right
                tasks[j + 1] = tmp;       // put the saved one in right's spot
            }
            j = j + 1;
        }
        i = i + 1;
    }
}
