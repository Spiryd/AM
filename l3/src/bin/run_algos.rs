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
        let mut best = 0;
        let mut best_sa = 0;
        let mut best_ts = 0;
        let mut avg = 0.;
        let mut avg_sa = 0.;
        let mut avg_ts = 0.;
        for _ in 0..100 {
            let sa = simulated_annealing(&adj_matrix, point_count, 5_000).1;
            avg += sa as f64 / 200.0;
            avg_sa += sa as f64 / 100.0;
            if sa < best {
                best = sa;
            }
            if sa < best_sa {
                best_sa = sa;
            }
            let ts = tabu_search(&adj_matrix,  (point_count * 3)/2).1;
            avg += ts as f64 / 200.0;
            avg_ts += ts as f64 / 100.0;
            if ts < best {
                best = ts;
            }
            if ts < best_ts {
                best_ts = ts;
            }
        }
        println!("best: {:?}", best);
        println!("best_sa: {:?}", best_sa);
        println!("best_ts: {:?}", best_ts);
        println!("avg: {:?}", avg);
        println!("avg_sa: {:?}", avg_sa);
        println!("avg_ts: {:?}", avg_ts);
    }
}
