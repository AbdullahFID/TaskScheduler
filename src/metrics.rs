// metrics.rs - tracks performance stats during the simulation
// this is like the scoreboard - it watches everything that happens and
// at the end it tells you how well the scheduler did
// did tasks wait too long? did we miss deadlines? was the cpu busy or idle?
// all that stuff gets tracked here

pub struct Metrics {
    pub total: i32,       // total number of tasks that were loaded into the simulation
    pub done: i32,        // how many tasks actually finished running
    pub missed: i32,      // how many tasks missed their deadline (finished too late)
    pub total_wait: i32,  // sum of all wait times across every task
                           // we add up every tasks wait time and divide by done to get the average later
    pub max_wait: i32,    // the longest any single task had to wait before it got to run
                           // this is the worst case scenario, the unluckiest task
    pub busy_time: i32,   // total seconds the cpu was actually running tasks (not idle)
                           // used to calculate cpu utilization at the end
    pub end_time: i32,    // what time the simulation clock was at when everything finished
                           // used for throughput calculation (tasks per second)
}

impl Metrics {
    // create a fresh metrics tracker with everything at zero
    pub fn new() -> Metrics {
        Metrics {
            total: 0,
            done: 0,
            missed: 0,
            total_wait: 0,
            max_wait: 0,
            busy_time: 0,
            end_time: 0,
        }
    }

    // call this every time a task starts running
    // it records the wait time, checks if the deadline was missed, and updates the counters
    //
    // arrival = when did this task first show up at the scheduler
    // start_time = what time is it NOW when we start running it
    // duration = how many seconds will it take to finish
    // abs_deadline = the absolute time this task must be done by (arrival + relative deadline)
    //
    // wait time = start_time - arrival (how long the task sat around before running)
    // example: task arrived at T=5, started running at T=20 -> wait = 15 seconds
    //
    // finish time = start_time + duration (when will it actually be done)
    // example: started at T=20, takes 8 seconds -> finishes at T=28
    // if deadline was T=25, then 28 > 25 = MISSED
    pub fn record_task(&mut self, arrival: i32, start_time: i32, duration: i32, abs_deadline: i32) {
        let wait = start_time - arrival;     // how long did this task wait?
        let finish = start_time + duration;  // when will it finish?

        self.done = self.done + 1;                // one more task completed
        self.total_wait = self.total_wait + wait; // add to running total for averaging later
        self.busy_time = self.busy_time + duration; // cpu was busy for this many seconds

        // track the worst wait time we've seen
        if wait > self.max_wait {
            self.max_wait = wait;
        }

        // check if we missed the deadline
        // if the task finishes AFTER its absolute deadline, thats a miss
        if finish > abs_deadline {
            self.missed = self.missed + 1;
        }
    }

    // print the final performance report at the end of the simulation
    // this is what you see after "=== SIMULATION COMPLETE ==="
    pub fn print_report(&self) {
        println!();
        println!("=== PERFORMANCE METRICS ===");

        // how many tasks finished out of how many were loaded
        println!("Tasks completed:  {} / {}", self.done, self.total);

        // how many tasks finished after their deadline
        println!("Deadlines missed: {}", self.missed);

        // average wait time = total wait across all tasks / number of tasks
        // this tells you on average how long a task sits in the queue before running
        if self.done > 0 {
            let avg_wait = self.total_wait as f64 / self.done as f64;
            println!("Avg wait time:    {:.1}s", avg_wait); // {:.1} means 1 decimal place
        } else {
            println!("Avg wait time:    N/A"); // cant divide by zero if no tasks finished
        }

        // the single longest wait any task experienced (worst case)
        println!("Max wait time:    {}s", self.max_wait);

        if self.end_time > 0 {
            // throughput = tasks completed per second of simulation time
            // higher is better, means we're getting through tasks quickly
            let throughput = self.done as f64 / self.end_time as f64;
            println!("Throughput:       {:.3} tasks/sec", throughput);

            // cpu utilization = what percentage of the total time was the cpu actually doing work
            // 100% means the cpu never had to idle (always had something to do)
            // lower means there were gaps where no tasks were ready
            let util = (self.busy_time as f64 / self.end_time as f64) * 100.0;
            println!("CPU utilization:  {:.1}%", util);
        } else {
            println!("Throughput:       N/A");
            println!("CPU utilization:  N/A");
        }

        println!("===========================");
    }
}
