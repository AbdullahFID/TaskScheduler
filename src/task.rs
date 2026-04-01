#![allow(dead_code)]
// task.rs - task struct and priority stuff

pub const MAX_TASKS: usize = 1024;
pub const MAX_NAME: usize = 64;

// priority levels (higher number = more urgent)
pub const PRI_LOW: i32 = 1;
pub const PRI_MED: i32 = 2;
pub const PRI_HIGH: i32 = 3;
pub const PRI_CRIT: i32 = 4;

// status codes for tracking where a task is
pub const ST_PENDING: i32 = 0;
pub const ST_QUEUED: i32 = 1;
pub const ST_READY: i32 = 2;
pub const ST_RUNNING: i32 = 3;
pub const ST_DONE: i32 = 4;

#[derive(Clone, Copy)] // <-- Rusts equievelnt to struct Task copy = original; we can copy this structs around by copying the bytes  
pub struct Task {
    pub id: i32,
    pub name: [u8; MAX_NAME], // i would have used strings but the prof said i cant i have to do everything manually
    pub name_len: usize,
    pub priority: i32,
    pub arrival: i32,
    pub deadline: i32,   // seconds after arrival
    pub duration: i32,
    pub status: i32,
    pub order: i32,      // arrival order for fifo tiebreaking
}

impl Task {
    //  Since Rust doesn't have null, we create a "blank" task with zeroed fields.
    pub fn empty() -> Task {
        Task {
            id: 0,
            name: [0u8; MAX_NAME],
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
    pub fn set_name(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let len = if bytes.len() > MAX_NAME { MAX_NAME } else { bytes.len() }; // cap it to max_name to prveent overflow
        let mut i = 0;
        while i < len { // this is quievlent to strncpy from rust docs 
            self.name[i] = bytes[i];
            i = i + 1;
        }
        self.name_len = len;
    }

    // get the name back as a string
    pub fn get_name(&self) -> &str {
        match std::str::from_utf8(&self.name[..self.name_len]) {
            Ok(s) => s, 
            Err(_) => "???", // if data is readable convert bytes to normal text else return ???
        }
    }

    // priority rank for heap ordering (lower = more urgent)
    // CRITICAL=0, HIGH=1, MED=2, LOW=3
    pub fn rank(&self) -> i32 {
        return 4 - self.priority;
    }

    // absolute deadline = arrival + relative deadline (so if item comes at T=5 and deadline in 30 then abs = 35)
    pub fn abs_deadline(&self) -> i32 {
        return self.arrival + self.deadline;
    }

    // get a nice string for the priority
    pub fn priority_str(&self) -> &str {
        if self.priority == PRI_CRIT { return "CRITICAL"; }
        if self.priority == PRI_HIGH { return "HIGH"; }
        if self.priority == PRI_MED { return "MEDIUM"; }
        return "LOW";
    }
}

// simple bubble sort for task array by arrival time
// O(n^2) def not the "best" but easiest for me, could have done quick sort but too hard and also wouldnt make much a difference if <- 1024 (max limit)
pub fn sort_by_arrival(tasks: &mut [Task], count: usize) {
    let mut i = 0;
    while i < count {
        let mut j = 0;
        while j < count - 1 - i {
            if tasks[j].arrival > tasks[j + 1].arrival {
                let tmp = tasks[j];
                tasks[j] = tasks[j + 1];
                tasks[j + 1] = tmp;
            }
            j = j + 1;
        }
        i = i + 1;
    }
}
