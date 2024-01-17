use std::time::Instant;
use std::sync::{Arc, Mutex};
use std::thread;
use std::fs::File;
use std::io::Write;
use l4::*;

fn main() {
    let points = file_to_points("test_data/1.tsp");
    let point_count = points.len();
    let adj_matrix = points_to_matrix(points);
    let avg_time: Arc<Mutex<f64>> = Arc::new(Mutex::new(0.0));
    let avg_weight: Arc<Mutex<f64>> = Arc::new(Mutex::new(0.0));
    let mut handles = Vec::new();
    for _ in 0..10 {
        let avg_time = Arc::clone(&avg_time);
        let avg_weight = Arc::clone(&avg_weight);
        let adj_matrix = adj_matrix.clone();
        let handle = thread::spawn(move || {
            let start = Instant::now();
            let mut ga = Evolution::new(4, point_count, adj_matrix);
            ga.run(true);
            let (_, weight) = ga.extract_best();
            let elapsed = start.elapsed().as_secs_f64();
            let mut time_acc = avg_time.lock().unwrap();
            let mut weight_acc = avg_weight.lock().unwrap();
            *time_acc += elapsed / 10.0;
            *weight_acc += weight as f64 / 10.0;
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    println!("Type: PMX, Weight: {}, Time: {}", avg_weight.lock().unwrap(), avg_time.lock().unwrap());
    
    let avg_time: Arc<Mutex<f64>> = Arc::new(Mutex::new(0.0)); 
    let avg_weight: Arc<Mutex<f64>> = Arc::new(Mutex::new(0.0));
    let mut handles = Vec::new();
    for _ in 0..10 {
        let avg_time = Arc::clone(&avg_time);
        let avg_weight = Arc::clone(&avg_weight);
        let adj_matrix = adj_matrix.clone();
        let handle = thread::spawn(move || {
            let start = Instant::now();
            let mut ga = Evolution::new(4, point_count, adj_matrix);
            ga.run(false);
            let (_, weight) = ga.extract_best();
            let elapsed = start.elapsed().as_secs_f64();
            let mut time_acc = avg_time.lock().unwrap();
            let mut weight_acc = avg_weight.lock().unwrap();
            *time_acc += elapsed / 10.0;
            *weight_acc += weight as f64 / 10.0;
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    println!("Type: CX, Weight: {}, Time: {}", avg_weight.lock().unwrap(), avg_time.lock().unwrap());

    let mut file = File::create("data.csv").expect("Failed to create file");
    file.write_all(b"map;avg_weight;avg_time\n").expect("Failed to write to file");
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
        "test_data/b.tsp",
        "test_data/c.tsp",
        "test_data/d.tsp",
        "test_data/e.tsp",
        "test_data/f.tsp",
    ] {
        let points = file_to_points(path);
        let point_count = points.len();
        let adj_matrix = points_to_matrix(points);

        let avg_time: Arc<Mutex<f64>> = Arc::new(Mutex::new(0.0));
        let avg_weight: Arc<Mutex<f64>> = Arc::new(Mutex::new(0.0));
        for _ in 0..10 {
            let mut handles = Vec::new();
            for _ in 0..10 {
                let avg_time = Arc::clone(&avg_time);
                let avg_weight = Arc::clone(&avg_weight);
                let adj_matrix = adj_matrix.clone();
                let handle = thread::spawn(move || {
                    let start = Instant::now();
                    let mut ga = Evolution::new(4, point_count, adj_matrix);
                    ga.run(true);
                    let (_, weight) = ga.extract_best();
                    let elapsed = start.elapsed().as_secs_f64();
                    let mut time_acc = avg_time.lock().unwrap();
                    let mut weight_acc = avg_weight.lock().unwrap();
                    *time_acc += elapsed / 100.0;
                    *weight_acc += weight as f64 / 100.0;
                });
                handles.push(handle);
            }
            for handle in handles {
                handle.join().unwrap();
            }
        }
        file.write_all(format!("{};{};{}\n", point_count, avg_weight.lock().unwrap(), avg_time.lock().unwrap()).as_bytes()).expect("Failed to write to file");
    }
}
