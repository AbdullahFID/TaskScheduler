// fsm.rs - finite state machine for the scheduler
// controls what the system is doing at any given time

// states
pub const STATE_IDLE: i32 = 0;      // nothing to do right now
pub const STATE_READY: i32 = 1;     // tasks available, picking next
pub const STATE_RUNNING: i32 = 2;   // currently executing a task
pub const STATE_SHUTDOWN: i32 = 3;  // all done

pub struct Fsm {
    pub state: i32,
}

impl Fsm {
    pub fn new() -> Fsm {
        Fsm {
            state: STATE_IDLE,
        }
    }

    // try to transition to a new state
    // returns 0 on success, -1 if the transition doesnt make sense
    pub fn transition(&mut self, new_state: i32) -> i32 {
        // check if this transition is valid
        let ok = match (self.state, new_state) {
            (STATE_IDLE, STATE_READY)     => true,  // tasks arrived
            (STATE_IDLE, STATE_SHUTDOWN)  => true,  // nothing left
            (STATE_READY, STATE_RUNNING)  => true,  // picked a task
            (STATE_READY, STATE_IDLE)     => true,  // heap drained mid-check
            (STATE_RUNNING, STATE_IDLE)   => true,  // task finished
            (STATE_RUNNING, STATE_READY)  => true,  // task done, more waiting
            _ => false,
        };

        if ok {
            self.state = new_state;
            return 0;
        } else {
            println!("  [fsm] bad transition {} -> {}", self.state_str(), state_name(new_state));
            return -1;
        }
    }

    pub fn state_str(&self) -> &str {
        return state_name(self.state);
    }
}

fn state_name(s: i32) -> &'static str {
    if s == STATE_IDLE { return "IDLE"; }
    if s == STATE_READY { return "READY"; }
    if s == STATE_RUNNING { return "RUNNING"; }
    if s == STATE_SHUTDOWN { return "SHUTDOWN"; }
    return "UNKNOWN";
}
