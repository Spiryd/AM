use l3::*;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn main() {
    for path in [
        "test_data/b.tsp",
        "test_data/c.tsp",
        "test_data/d.tsp",
        "test_data/e.tsp",
        "test_data/f.tsp",
    ] {
        for _ in 0..100 {
            let points = file_to_points(path);
            let point_count = points.len();
            let adj_matrix = points_to_matrix(points);
            let x = vec![0,1,2,3,4,5,6,7,8,9];
            calculate_hash(&x);
            simulated_annealing(&adj_matrix, point_count, 1_000);
            tabu_search(&adj_matrix, point_count);
        }
    }
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}