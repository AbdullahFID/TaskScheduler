// fsm.rs - finite state machine for the scheduler
// controls what the system is doing at any given time
// think of it like a traffic light - it can only be in certain states
// and it can only change between states in specific valid ways
//
// the whole point of this is to be a safety net for the simulation logic
// if our code ever tries to do something impossible (like go from IDLE straight to RUNNING
// without checking whats ready first) the FSM catches it and yells at us
// its like a bouncer at a club checking if youre allowed to go from one room to another
//
// the valid flow is:
//   IDLE -> READY -> RUNNING -> IDLE -> READY -> RUNNING -> ... -> SHUTDOWN
//
// IDLE means "nothing happening, waiting for work"
// READY means "tasks are available, picking the best one"
// RUNNING means "executing a task right now"
// SHUTDOWN means "all done, simulation is over"

// states - just integers so we can compare them easily
pub const STATE_IDLE: i32 = 0;      // cpu is chilling, nothing to do right now
pub const STATE_READY: i32 = 1;     // tasks are in the heap, about to pick one
pub const STATE_RUNNING: i32 = 2;   // currently executing a task
pub const STATE_SHUTDOWN: i32 = 3;  // everything is done, simulation over, go home

// the FSM struct is dead simple - just holds the current state as an integer
pub struct Fsm {
    pub state: i32,
}

impl Fsm {
    // create a new FSM starting in the IDLE state
    // the scheduler always starts idle before any tasks arrive
    pub fn new() -> Fsm {
        Fsm {
            state: STATE_IDLE,
        }
    }

    // try to transition to a new state
    // this is where the "rules" are enforced
    // returns 0 if the transition is valid and happened, -1 if its not allowed
    //
    // valid transitions and what they mean:
    //   IDLE -> READY      tasks showed up, time to pick one
    //   IDLE -> SHUTDOWN   no tasks left at all, were done
    //   READY -> RUNNING   we picked a task from the heap, start executing it
    //   READY -> IDLE      false alarm, heap got emptied somehow, go back to waiting
    //   RUNNING -> IDLE    task finished, nothing else to do right now
    //   RUNNING -> READY   task finished, but theres more tasks waiting
    //
    // anything else is INVALID, for example:
    //   IDLE -> RUNNING     you cant run a task without checking whats ready first
    //   SHUTDOWN -> anything once youre shutdown youre done, no coming back
    //   RUNNING -> SHUTDOWN you gotta go back to IDLE first
    pub fn transition(&mut self, new_state: i32) -> i32 {
        // check if this transition is allowed using pattern matching
        // match (current_state, requested_state) against the allowed pairs
        let ok = match (self.state, new_state) {
            (STATE_IDLE, STATE_READY)     => true,  // tasks arrived
            (STATE_IDLE, STATE_SHUTDOWN)  => true,  // nothing left
            (STATE_READY, STATE_RUNNING)  => true,  // picked a task
            (STATE_READY, STATE_IDLE)     => true,  // heap drained mid-check
            (STATE_RUNNING, STATE_IDLE)   => true,  // task finished
            (STATE_RUNNING, STATE_READY)  => true,  // task done, more waiting
            _ => false,  // everything else is not allowed
        };

        if ok {
            self.state = new_state; // update the state
            return 0;               // success
        } else {
            // this should never happen if the simulation logic is correct
            // if it does, we have a bug somewhere
            println!("  [fsm] bad transition {} -> {}", self.state_str(), state_name(new_state));
            return -1;
        }
    }

    // get the current state as a human readable string for printing
    pub fn state_str(&self) -> &str {
        return state_name(self.state);
    }
}

// helper function that converts a state integer to its name
// used for printing so the output says "IDLE" instead of "0"
fn state_name(s: i32) -> &'static str {
    if s == STATE_IDLE { return "IDLE"; }
    if s == STATE_READY { return "READY"; }
    if s == STATE_RUNNING { return "RUNNING"; }
    if s == STATE_SHUTDOWN { return "SHUTDOWN"; }
    return "UNKNOWN"; // should never happen but just in case
}
