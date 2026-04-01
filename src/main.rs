// main.rs - entry point for the task scheduler simulator
// this is where everything comes together
// it reads tasks from a csv file (or generates random ones), runs the scheduling simulation,
// and prints out the results at the end
//
// the whole pipeline is:
//   CSV/Generator -> [Task array] -> sort by arrival -> simulation loop -> final report
//
// usage:
//   cargo run -- <csv_file>              run simulation on a csv file
//   cargo run -- --generate <count>      generate random tasks and run
//   cargo run -- --generate-burst <cnt>  generate burst (all arrive at T=0)
//   cargo run -- --generate-overload <n> generate overload (tight deadlines)

// these "mod" lines tell rust to include the other files as modules
// each one is a separate file in the src/ folder
mod task;       // task struct and priority definitions
mod queue;      // circular queue for arrival buffer
mod heap;       // min-heap for priority scheduling
mod hashtable;  // hash table for fast task lookup by id
mod fsm;        // finite state machine for system state tracking
mod metrics;    // performance tracking and reporting
mod generator;  // random task generator for stress testing

// import the specific things we need from each module
// this way we can write Queue::new() instead of queue::Queue::new() everywhere
use task::{Task, MAX_TASKS, ST_QUEUED, ST_READY, ST_RUNNING, ST_DONE};
use queue::Queue;
use heap::{Heap, HeapEntry};
use hashtable::HashTable;
use fsm::{Fsm, STATE_IDLE, STATE_READY, STATE_RUNNING, STATE_SHUTDOWN};
use metrics::Metrics;

// parse a csv file and load tasks into the array
// the csv format is: id,name,priority,arrival,deadline,duration
// first row is the header (we skip it)
// returns how many tasks were successfully loaded
// if anything goes wrong with a row (bad data, missing fields) we skip that row and keep going
fn load_csv(path: &str, tasks: &mut [Task; MAX_TASKS]) -> usize {
    // read the entire file into a string
    // if the file doesnt exist or cant be read we print an error and return 0
    let content = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            println!("error reading {}: {}", path, e);
            return 0;
        }
    };

    let mut count: usize = 0;
    let mut first_line = 1; // flag to skip the header row (id,name,priority,arrival,deadline,duration)

    // go through each line of the file
    for line in content.split('\n') {
        let trimmed = line.trim();

        // skip empty lines (blank lines in the csv)
        if trimmed.is_empty() {
            continue;
        }
        // skip the header row (first non-empty line)
        if first_line == 1 {
            first_line = 0;
            continue;
        }

        // safety check: dont go over our max task limit
        if count >= MAX_TASKS {
            println!("  warning: hit max tasks ({}), ignoring rest", MAX_TASKS);
            break;
        }

        // split the line by commas to get each field
        // csv format: id,name,priority,arrival,deadline,duration
        let mut parts = trimmed.split(',');
        // try to grab each field, if any are missing skip this row
        let id_str = match parts.next() { Some(s) => s.trim(), None => continue };
        let name_str = match parts.next() { Some(s) => s.trim(), None => continue };
        let pri_str = match parts.next() { Some(s) => s.trim(), None => continue };
        let arr_str = match parts.next() { Some(s) => s.trim(), None => continue };
        let dead_str = match parts.next() { Some(s) => s.trim(), None => continue };
        let dur_str = match parts.next() { Some(s) => s.trim(), None => continue };

        // try to parse each string field into a number
        // if any of them fail to parse (like if someone put "abc" where a number should be)
        // we print a warning and skip the entire row
        let id: i32 = match id_str.parse() { Ok(v) => v, Err(_) => {
            println!("  warning: bad id '{}', skipping row", id_str);
            continue;
        }};
        let pri: i32 = match pri_str.parse() { Ok(v) => v, Err(_) => {
            println!("  warning: bad priority '{}', skipping row", pri_str);
            continue;
        }};
        let arr: i32 = match arr_str.parse() { Ok(v) => v, Err(_) => {
            println!("  warning: bad arrival '{}', skipping row", arr_str);
            continue;
        }};
        let dead: i32 = match dead_str.parse() { Ok(v) => v, Err(_) => {
            println!("  warning: bad deadline '{}', skipping row", dead_str);
            continue;
        }};
        let dur: i32 = match dur_str.parse() { Ok(v) => v, Err(_) => {
            println!("  warning: bad duration '{}', skipping row", dur_str);
            continue;
        }};

        // validate the parsed values make sense
        // these are sanity checks to catch obviously wrong data
        if id <= 0 {
            println!("  warning: id must be positive (got {}), skipping", id);
            continue;
        }
        if pri < 1 || pri > 4 {
            println!("  warning: priority must be 1-4 (got {}), skipping", pri);
            continue;
        }
        if arr < 0 {
            println!("  warning: arrival cant be negative (got {}), skipping", arr);
            continue;
        }
        if dead <= 0 {
            println!("  warning: deadline must be positive (got {}), skipping", dead);
            continue;
        }
        if dur <= 0 {
            println!("  warning: duration must be positive (got {}), skipping", dur);
            continue;
        }

        // all checks passed, build the task and add it to the array
        let mut t = Task::empty();
        t.id = id;
        t.set_name(name_str);
        t.priority = pri;
        t.arrival = arr;
        t.deadline = dead;
        t.duration = dur;
        t.order = count as i32; // arrival order for FIFO tiebreaking in the heap

        tasks[count] = t;
        count = count + 1;
    }

    return count;
}

// the main scheduling simulation - this is where all the data structures work together
// takes the task array and runs the full simulation loop
//
// the simulation works like this:
//   1. sort tasks by arrival time (so we process them in chronological order)
//   2. set up all our data structures (queue, heap, hash table, fsm, metrics)
//   3. run a loop where each iteration:
//      a. PHASE 1 - check if any new tasks have arrived at the current time
//      b. PHASE 2 - move arrived tasks from the queue into the priority heap
//      c. PHASE 3 - pick the most urgent task from the heap and run it
//   4. repeat until all tasks are done
//   5. print the performance report
fn run_simulation(tasks: &mut [Task; MAX_TASKS], num_tasks: usize) {
    // sort tasks by arrival time so we can process them in order
    // this way we just need a pointer (pending_idx) that moves forward through the array
    task::sort_by_arrival(tasks, num_tasks);

    // set up all our data structures
    let mut queue = Queue::new();       // arrival buffer - tasks go here when they first show up
    let mut ready_heap = Heap::new();   // priority queue - sorts tasks by urgency
    let mut ht = HashTable::new();      // lookup table - stores full task details by id
    let mut fsm = Fsm::new();           // state machine - tracks IDLE/READY/RUNNING/SHUTDOWN
    let mut stats = Metrics::new();     // performance tracker - records wait times, misses, etc

    stats.total = num_tasks as i32;

    let mut sim_time: i32 = 0;          // the simulation clock, starts at 0 seconds
    let mut pending_idx: usize = 0;     // index into the sorted task array
                                         // tracks which tasks havent arrived yet
                                         // everything before this index has already been enqueued

    println!();
    println!("=== SIMULATION START ===");
    println!("loaded {} tasks", num_tasks);
    println!();

    // main simulation loop - keeps going until all tasks are processed
    loop {
        // PHASE 1: check for new arrivals
        // any task whose arrival time <= current sim time gets put into the queue
        // we check tasks in order (sorted by arrival) so once we hit a task that hasnt
        // arrived yet, we know all remaining tasks havent arrived either and we can stop
        while pending_idx < num_tasks && tasks[pending_idx].arrival <= sim_time {
            let t = tasks[pending_idx];
            queue.enqueue(t.id); // put the task id into the arrival queue

            // also register the full task in the hash table so we can look it up later
            let mut t_copy = t;
            t_copy.status = ST_QUEUED; // mark it as queued
            ht.insert(t_copy);

            pending_idx = pending_idx + 1; // move to the next task
        }

        // PHASE 2: drain the queue into the ready heap
        // take every task id out of the queue, look up its details in the hash table,
        // create a lightweight HeapEntry with just the sorting fields, and insert into the heap
        // after this phase the heap has all arrived tasks sorted by urgency
        while !queue.is_empty() {
            let tid = queue.dequeue(); // grab next task id from the queue
            if tid < 0 {
                break; // shouldnt happen but just in case
            }

            // look up the full task details in the hash table
            if let Some(t) = ht.lookup(tid) {
                // create a heap entry with just the fields needed for priority comparison
                let entry = HeapEntry {
                    rank: t.rank(),           // priority rank (0=CRITICAL, 3=LOW)
                    deadline: t.abs_deadline(), // absolute deadline for tiebreaking
                    duration: t.duration,      // duration for shortest-job tiebreaking
                    order: t.order,            // arrival order for FIFO tiebreaking
                    task_id: t.id,             // so we can look up full details later
                };
                ready_heap.insert(entry);        // add to the priority heap
                ht.update_status(tid, ST_READY); // update status: QUEUED -> READY
            }
        }

        // PHASE 3: schedule and run the next task
        if !ready_heap.is_empty() {
            // theres at least one task ready to run
            fsm.transition(STATE_READY); // tell the FSM we have tasks available

            // extract the most urgent task from the heap (the root)
            let entry = ready_heap.extract_min();
            // look up the full task details from the hash table
            let task_info = ht.lookup(entry.task_id);

            if let Some(t) = task_info {
                // start running this task
                fsm.transition(STATE_RUNNING);          // FSM: READY -> RUNNING
                ht.update_status(t.id, ST_RUNNING);     // hash table: mark as running

                // print what we're doing
                println!(
                    "[T={:>4}s]  TASK {:>3} ({:<8}) scheduled  — {}",
                    sim_time,            // current clock time
                    t.id,                // task id
                    t.priority_str(),    // "CRITICAL", "HIGH", etc
                    t.get_name(),        // task name like "pick_up_box"
                );

                // record this task's metrics (wait time, deadline check, etc)
                stats.record_task(t.arrival, sim_time, t.duration, t.abs_deadline());

                // advance the simulation clock by how long this task takes
                // the cpu is "busy" for this many seconds
                sim_time = sim_time + t.duration;

                // check if the task finished after its deadline
                let missed = if sim_time > t.abs_deadline() { " [MISSED]" } else { "" };

                println!(
                    "[T={:>4}s]  TASK {:>3} complete{}",
                    sim_time, t.id, missed,
                );

                // task is done, update its status and go back to IDLE
                ht.update_status(t.id, ST_DONE);   // hash table: mark as done
                fsm.transition(STATE_IDLE);          // FSM: RUNNING -> IDLE
            }
        } else if pending_idx < num_tasks {
            // no tasks are ready RIGHT NOW, but more tasks will arrive in the future
            // instead of sitting here doing nothing, jump the clock forward to the next arrival
            // this is an optimization so we dont loop through empty time ticks
            sim_time = tasks[pending_idx].arrival;
        } else {
            // no tasks in the heap AND no more tasks coming = we're done
            fsm.transition(STATE_SHUTDOWN); // FSM: IDLE -> SHUTDOWN
            break; // exit the main loop
        }
    }

    // record when the simulation ended (for throughput calculation)
    stats.end_time = sim_time;

    println!();
    println!("=== SIMULATION COMPLETE === (state: {})", fsm.state_str());

    // print the final performance report (avg wait, missed deadlines, throughput, etc)
    stats.print_report();
}

// print usage instructions when no arguments are given
fn print_usage() {
    println!("task scheduler simulator - MREN 178 project");
    println!();
    println!("usage:");
    println!("  task_scheduler <csv_file>              run on csv input");
    println!("  task_scheduler --generate <count>      generate random tasks");
    println!("  task_scheduler --generate-burst <cnt>  generate burst load");
    println!("  task_scheduler --generate-overload <n> generate overload scenario");
}

// main() - the very first function that runs when you do cargo run
// parses command line arguments and decides what mode to run in:
//   1. load tasks from a csv file
//   2. generate random tasks (normal spread, burst, or overload)
// then passes everything to run_simulation()
fn main() {
    // collect command line args into a vector of strings
    // args[0] is the program name itself, args[1] is the first real argument
    let args: Vec<String> = std::env::args().collect();

    // if no arguments provided, print help and exit
    if args.len() < 2 {
        print_usage();
        return;
    }

    // create the task array - starts as 1024 blank tasks
    // we'll fill it with real data below
    let mut tasks = [Task::empty(); MAX_TASKS];
    let num_tasks: usize;

    // check what mode the user wants
    let mode = args[1].as_str();

    if mode == "--generate" || mode == "--generate-burst" || mode == "--generate-overload" {
        // GENERATE MODE: create random tasks in memory

        // figure out how many tasks to generate (default 50 if not specified)
        let count: usize = if args.len() >= 3 {
            match args[2].parse() {
                Ok(n) => n,
                Err(_) => {
                    println!("bad count: {}", args[2]);
                    return;
                }
            }
        } else {
            50 // default count
        };

        let seed = 12345u64; // fixed seed so results are the same every run (reproducible)

        if mode == "--generate-burst" {
            // BURST MODE: all tasks arrive at T=0
            // stress test for the heap - has to sort everything at once
            println!("generating {} tasks (burst mode - all at T=0)...", count);
            num_tasks = generator::generate_tasks(&mut tasks, count, 0, seed);
        } else if mode == "--generate-overload" {
            // OVERLOAD MODE: tasks with impossibly tight deadlines
            // tests how the system handles missed deadlines gracefully
            println!("generating {} tasks (overload - tight deadlines)...", count);
            num_tasks = generator::generate_tasks(&mut tasks, count, 10, seed);
            // make deadlines barely possible - duration + 2 seconds of slack
            // most tasks WILL miss their deadline, thats the point
            let mut i = 0;
            while i < num_tasks {
                tasks[i].deadline = tasks[i].duration + 2;
                i = i + 1;
            }
        } else {
            // NORMAL MODE: tasks spread across time randomly
            // the most realistic scenario
            println!("generating {} tasks (normal spread)...", count);
            num_tasks = generator::generate_tasks(&mut tasks, count, 100, seed);
        }
    } else {
        // CSV MODE: load tasks from a file
        println!("loading tasks from {}...", mode);
        num_tasks = load_csv(mode, &mut tasks);
    }

    // if we didnt load any tasks, nothing to simulate
    if num_tasks == 0 {
        println!("no valid tasks loaded, nothing to do");
        return;
    }

    // run the actual simulation
    run_simulation(&mut tasks, num_tasks);
}
