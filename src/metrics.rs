// metrics.rs - tracks performance stats during the simulation

pub struct Metrics {
    pub total: i32,       // total tasks loaded
    pub done: i32,        // tasks that finished
    pub missed: i32,      // tasks that missed their deadline
    pub total_wait: i32,  // sum of all wait times (for computing average)
    pub max_wait: i32,    // longest any task had to wait
    pub busy_time: i32,   // total time the cpu was running tasks
    pub end_time: i32,    // sim time when everything finished
}

impl Metrics {
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

    // call this when a task starts running
    pub fn record_task(&mut self, arrival: i32, start_time: i32, duration: i32, abs_deadline: i32) {
        let wait = start_time - arrival;
        let finish = start_time + duration;

        self.done = self.done + 1;
        self.total_wait = self.total_wait + wait;
        self.busy_time = self.busy_time + duration;

        if wait > self.max_wait {
            self.max_wait = wait;
        }

        // check if we missed the deadline
        if finish > abs_deadline {
            self.missed = self.missed + 1;
        }
    }

    // print the final report
    pub fn print_report(&self) {
        println!();
        println!("=== PERFORMANCE METRICS ===");
        println!("Tasks completed:  {} / {}", self.done, self.total);
        println!("Deadlines missed: {}", self.missed);

        if self.done > 0 {
            let avg_wait = self.total_wait as f64 / self.done as f64;
            println!("Avg wait time:    {:.1}s", avg_wait);
        } else {
            println!("Avg wait time:    N/A");
        }

        println!("Max wait time:    {}s", self.max_wait);

        if self.end_time > 0 {
            let throughput = self.done as f64 / self.end_time as f64;
            println!("Throughput:       {:.3} tasks/sec", throughput);

            let util = (self.busy_time as f64 / self.end_time as f64) * 100.0;
            println!("CPU utilization:  {:.1}%", util);
        } else {
            println!("Throughput:       N/A");
            println!("CPU utilization:  N/A");
        }

        println!("===========================");
    }
}
