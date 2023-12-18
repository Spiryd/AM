use std::time::Instant;
use std::sync::{Arc, Mutex};
use std::thread;

use l3::*;

fn main() {
    for path in [
        "test_data/1.tsp",
        "test_data/2.tsp",
        "test_data/3.tsp",
        "test_data/4.tsp",
        "test_data/5.tsp",
        "test_data/6.tsp",
        "test_data/7.tsp",
        "test_data/8.tsp",
        "test_data/9.tsp",
        "test_data/a.tsp",
    ] {
        let mut results_sa: Vec<(usize, usize, usize, usize, f64)> = Vec::new();
        let points = file_to_points(path);
        let point_count = points.len();
        let adj_matrix = points_to_matrix(points);
        for temp in 1..=4 {
            for epoch_count in (500..=5_000).step_by(500){
                let mut handles = Vec::new();
                let time_acc: Arc<Mutex<f64>> = Arc::new(Mutex::new(0.0));
                let weight_acc: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
                for _ in 0..10 {
                    let time_acc = Arc::clone(&time_acc);
                    let weight_acc = Arc::clone(&weight_acc);
                    let adj_matrix = adj_matrix.clone();
                    let handle = thread::spawn(move || {
                        let mut time = time_acc.lock().unwrap();
                        let mut weight = weight_acc.lock().unwrap();
                        let before = Instant::now();
                        let temperature = ((point_count * temp) as f64 * 0.5) as usize;
                        let sa = simulated_annealing(&adj_matrix, temperature, epoch_count);
                        *time += before.elapsed().as_secs_f64();
                        *weight += sa.1;
                    });
                    handles.push(handle);
                }
                for handle in handles {
                    handle.join().unwrap();
                }
                results_sa.push((point_count, temp, epoch_count, *weight_acc.clone().lock().unwrap()/10, *time_acc.clone().lock().unwrap()/10.0));
            }
        }
        println!("{:?}", results_sa.iter().min_by_key(|x| x.3).unwrap());
        let mut results_ts: Vec<(usize, usize, usize, f64)> = Vec::new();
        for tabu in 1..=32 {
            let mut handles = Vec::new();
            let time_acc: Arc<Mutex<f64>> = Arc::new(Mutex::new(0.0));
            let weight_acc: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
            for _ in 0..10 {
                let time_acc = Arc::clone(&time_acc);
                let weight_acc = Arc::clone(&weight_acc);
                let adj_matrix = adj_matrix.clone();
                let handle = thread::spawn(move || {
                    let mut time = time_acc.lock().unwrap();
                    let mut weight = weight_acc.lock().unwrap();
                    let before = Instant::now();
                    let tabu_capacity = ((point_count * tabu) as f64 * 0.0625) as usize;
                    let sa = tabu_search(&adj_matrix, tabu_capacity);
                    *time += before.elapsed().as_secs_f64();
                    *weight += sa.1;
                });
                handles.push(handle);
            }
            for handle in handles {
                handle.join().unwrap();
            }
            results_ts.push((point_count, tabu, *weight_acc.clone().lock().unwrap()/10, *time_acc.clone().lock().unwrap()/10.0))
        }
        println!("{:?}", results_ts.iter().min_by_key(|x| x.2).unwrap());
    }
}

