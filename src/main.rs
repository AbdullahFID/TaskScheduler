// main.rs - entry point for the task scheduler simulator
// reads tasks from csv, runs the scheduling simulation, prints results
//
// usage:
//   cargo run -- <csv_file>              run simulation on a csv file
//   cargo run -- --generate <count>      generate random tasks and run
//   cargo run -- --generate-burst <cnt>  generate burst (all arrive at T=0)

mod task;
mod queue;
mod heap;
mod hashtable;
mod fsm;
mod metrics;
mod generator;

use task::{Task, MAX_TASKS, ST_QUEUED, ST_READY, ST_RUNNING, ST_DONE};
use queue::Queue;
use heap::{Heap, HeapEntry};
use hashtable::HashTable;
use fsm::{Fsm, STATE_IDLE, STATE_READY, STATE_RUNNING, STATE_SHUTDOWN};
use metrics::Metrics;

// parse a csv file and load tasks into the array
// returns how many tasks were loaded
fn load_csv(path: &str, tasks: &mut [Task; MAX_TASKS]) -> usize {
    let content = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            println!("error reading {}: {}", path, e);
            return 0;
        }
    };

    let mut count: usize = 0;
    let mut first_line = 1; // skip header

    for line in content.split('\n') {
        let trimmed = line.trim();

        // skip empty lines and header
        if trimmed.is_empty() {
            continue;
        }
        if first_line == 1 {
            first_line = 0;
            continue;
        }

        if count >= MAX_TASKS {
            println!("  warning: hit max tasks ({}), ignoring rest", MAX_TASKS);
            break;
        }

        // parse: id,name,priority,arrival,deadline,duration
        let mut parts = trimmed.split(',');
        let id_str = match parts.next() { Some(s) => s.trim(), None => continue };
        let name_str = match parts.next() { Some(s) => s.trim(), None => continue };
        let pri_str = match parts.next() { Some(s) => s.trim(), None => continue };
        let arr_str = match parts.next() { Some(s) => s.trim(), None => continue };
        let dead_str = match parts.next() { Some(s) => s.trim(), None => continue };
        let dur_str = match parts.next() { Some(s) => s.trim(), None => continue };

        // parse numbers - skip row if anything fails
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

        // validate the values
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

        // build the task
        let mut t = Task::empty();
        t.id = id;
        t.set_name(name_str);
        t.priority = pri;
        t.arrival = arr;
        t.deadline = dead;
        t.duration = dur;
        t.order = count as i32;

        tasks[count] = t;
        count = count + 1;
    }

    return count;
}

// the main scheduling simulation
fn run_simulation(tasks: &mut [Task; MAX_TASKS], num_tasks: usize) {
    // sort tasks by arrival time first
    task::sort_by_arrival(tasks, num_tasks);

    // set up all our data structures
    let mut queue = Queue::new();
    let mut ready_heap = Heap::new();
    let mut ht = HashTable::new();
    let mut fsm = Fsm::new();
    let mut stats = Metrics::new();

    stats.total = num_tasks as i32;

    let mut sim_time: i32 = 0;
    let mut pending_idx: usize = 0; // tracks which tasks havent arrived yet

    println!();
    println!("=== SIMULATION START ===");
    println!("loaded {} tasks", num_tasks);
    println!();

    // main loop
    loop {
        // phase 1: check for new arrivals
        // any task whose arrival time <= current sim time goes into the queue
        while pending_idx < num_tasks && tasks[pending_idx].arrival <= sim_time {
            let t = tasks[pending_idx];
            queue.enqueue(t.id);

            // register in hash table with QUEUED status
            let mut t_copy = t;
            t_copy.status = ST_QUEUED;
            ht.insert(t_copy);

            pending_idx = pending_idx + 1;
        }

        // phase 2: drain the queue into the ready heap
        while !queue.is_empty() {
            let tid = queue.dequeue();
            if tid < 0 {
                break;
            }

            // look up the task to get its scheduling info
            if let Some(t) = ht.lookup(tid) {
                let entry = HeapEntry {
                    rank: t.rank(),
                    deadline: t.abs_deadline(),
                    duration: t.duration,
                    order: t.order,
                    task_id: t.id,
                };
                ready_heap.insert(entry);
                ht.update_status(tid, ST_READY);
            }
        }

        // phase 3: schedule the next task
        if !ready_heap.is_empty() {
            fsm.transition(STATE_READY);

            let entry = ready_heap.extract_min();
            let task_info = ht.lookup(entry.task_id);

            if let Some(t) = task_info {
                // start running this task
                fsm.transition(STATE_RUNNING);
                ht.update_status(t.id, ST_RUNNING);

                println!(
                    "[T={:>4}s]  TASK {:>3} ({:<8}) scheduled  — {}",
                    sim_time,
                    t.id,
                    t.priority_str(),
                    t.get_name(),
                );

                // record metrics
                stats.record_task(t.arrival, sim_time, t.duration, t.abs_deadline());

                // advance time by task duration
                sim_time = sim_time + t.duration;

                // check if deadline was missed
                let missed = if sim_time > t.abs_deadline() { " [MISSED]" } else { "" };

                println!(
                    "[T={:>4}s]  TASK {:>3} complete{}",
                    sim_time, t.id, missed,
                );

                ht.update_status(t.id, ST_DONE);
                fsm.transition(STATE_IDLE);
            }
        } else if pending_idx < num_tasks {
            // no tasks ready but more are coming - jump to next arrival
            sim_time = tasks[pending_idx].arrival;
        } else {
            // nothing left to do
            fsm.transition(STATE_SHUTDOWN);
            break;
        }
    }

    stats.end_time = sim_time;

    println!();
    println!("=== SIMULATION COMPLETE === (state: {})", fsm.state_str());

    stats.print_report();
}

fn print_usage() {
    println!("task scheduler simulator - MREN 178 project");
    println!();
    println!("usage:");
    println!("  task_scheduler <csv_file>              run on csv input");
    println!("  task_scheduler --generate <count>      generate random tasks");
    println!("  task_scheduler --generate-burst <cnt>  generate burst load");
    println!("  task_scheduler --generate-overload <n> generate overload scenario");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    let mut tasks = [Task::empty(); MAX_TASKS];
    let num_tasks: usize;

    let mode = args[1].as_str();

    if mode == "--generate" || mode == "--generate-burst" || mode == "--generate-overload" {
        // figure out count
        let count: usize = if args.len() >= 3 {
            match args[2].parse() {
                Ok(n) => n,
                Err(_) => {
                    println!("bad count: {}", args[2]);
                    return;
                }
            }
        } else {
            50 // default
        };

        let seed = 12345u64; // fixed seed so results are reproducible

        if mode == "--generate-burst" {
            // all tasks arrive at T=0
            println!("generating {} tasks (burst mode - all at T=0)...", count);
            num_tasks = generator::generate_tasks(&mut tasks, count, 0, seed);
        } else if mode == "--generate-overload" {
            // tight deadlines that cant all be met
            println!("generating {} tasks (overload - tight deadlines)...", count);
            num_tasks = generator::generate_tasks(&mut tasks, count, 10, seed);
            // make deadlines really tight
            let mut i = 0;
            while i < num_tasks {
                tasks[i].deadline = tasks[i].duration + 2; // barely enough time
                i = i + 1;
            }
        } else {
            // normal spread
            println!("generating {} tasks (normal spread)...", count);
            num_tasks = generator::generate_tasks(&mut tasks, count, 100, seed);
        }
    } else {
        // load from csv
        println!("loading tasks from {}...", mode);
        num_tasks = load_csv(mode, &mut tasks);
    }

    if num_tasks == 0 {
        println!("no valid tasks loaded, nothing to do");
        return;
    }

    run_simulation(&mut tasks, num_tasks);
}
