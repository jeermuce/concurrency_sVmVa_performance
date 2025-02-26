use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

// Simulate CPU-bound work
fn compute_intensive_task(iterations: usize, source: &str) -> usize {
    let mut result = 0;
    println!("{source}");
    for i in 0..iterations {
        // Some meaningless but CPU-intensive calculation
        result += (2 * (result + i * i * i + 1)) % 1_000_000_007;
    }
    result
}

fn single_threaded(total_work: usize, work_size: usize) -> Duration {
    let start = Instant::now();

    let mut final_result = 0;
    for _ in 0..(total_work / work_size) {
        final_result += compute_intensive_task(work_size, "SINGLE");
    }

    println!("Single-threaded result: {}", final_result);
    start.elapsed()
}

fn mutex_threads(total_work: usize, work_size: usize, n_threads: usize) -> Duration {
    let start = Instant::now();

    let result = Arc::new(Mutex::new(0));
    let chunks_per_thread = (total_work / work_size) / n_threads;

    let mut handles = vec![];

    for i in 0..n_threads {
        let result_clone = Arc::clone(&result);
        let handle = thread::spawn(move || {
            let mut local_sum = 0;
            for j in 0..chunks_per_thread {
                local_sum +=
                    compute_intensive_task(work_size, &format!("mutex: {i} thread || {j} chunk"));
            }

            // Only lock once at the end to add the thread's results
            let mut shared_result = result_clone.lock().unwrap();
            *shared_result += local_sum;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Mutex-threaded result: {}", *result.lock().unwrap());
    start.elapsed()
}

fn atomic_threads(total_work: usize, work_size: usize, n_threads: usize) -> Duration {
    let start = Instant::now();

    let result = Arc::new(AtomicUsize::new(0));
    let chunks_per_thread = (total_work / work_size) / n_threads;

    let mut handles = vec![];

    for i in 0..n_threads {
        let result_clone = Arc::clone(&result);
        let handle = thread::spawn(move || {
            let mut local_sum = 0;
            for j in 0..chunks_per_thread {
                local_sum +=
                    compute_intensive_task(work_size, &format!("atomic: {i} thread || {j} chunk"));
            }

            // Add this thread's results to the atomic counter
            result_clone.fetch_add(local_sum, Ordering::SeqCst);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Atomic-threaded result: {}", result.load(Ordering::SeqCst));
    start.elapsed()
}

fn main() {
    // Parameters
    let total_work = 10_000_000; // Total amount of work to do
    let work_size = 1000; // Size of each work chunk
    let n_threads = 4; // Number of threads to use (adjust to your CPU core count)

    // Run and time each implementation
    println!(
        "Running with total_work={}, work_size={}, threads={}",
        total_work, work_size, n_threads
    );

    let single_time = single_threaded(total_work, work_size);
    println!("Single-threaded time: {:?}", single_time);

    let mutex_time = mutex_threads(total_work, work_size, n_threads);
    println!("Mutex-threaded time: {:?}", mutex_time);
    println!(
        "Speedup vs single-threaded: {:.2}x",
        single_time.as_secs_f64() / mutex_time.as_secs_f64()
    );

    let atomic_time = atomic_threads(total_work, work_size, n_threads);
    println!("Atomic-threaded time: {:?}", atomic_time);
    println!(
        "Speedup vs single-threaded: {:.2}x",
        single_time.as_secs_f64() / atomic_time.as_secs_f64()
    );
    println!(
        "Speedup vs mutex: {:.2}x",
        mutex_time.as_secs_f64() / atomic_time.as_secs_f64()
    );
}
