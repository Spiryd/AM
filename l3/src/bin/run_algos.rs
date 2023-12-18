use l3::*;

fn main() {
    for path in [
        "test_data/b.tsp",
        "test_data/c.tsp",
        "test_data/d.tsp",
        "test_data/e.tsp",
        "test_data/f.tsp",
    ] {
        let points = file_to_points(path);
        let point_count = points.len();
        let adj_matrix = points_to_matrix(points);
        let mut best_sa = usize::MAX;
        let mut best_ts = usize::MAX;
        let mut avg_sa = 0.;
        let mut avg_ts = 0.;
        println!("map: {:?}", point_count);
        for _ in 0..100 {
            let sa = simulated_annealing(&adj_matrix, point_count/2, 5000).1;
            avg_sa += sa as f64 / 100.0;
            if sa < best_sa {
                best_sa = sa;
            }

            let ts = tabu_search(&adj_matrix,  point_count/2).1;
            avg_ts += ts as f64 / 100.0;
            if ts < best_ts {
                best_ts = ts;
            }
        }
        println!("best_ts: {:?}", best_ts);
        println!("avg_ts: {:?}", avg_ts);
        println!("best_sa: {:?}", best_sa);
        println!("avg_sa: {:?}", avg_sa);
    }
}
