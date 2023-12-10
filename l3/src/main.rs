use std::f64::consts::E;
use std::fs::{self, File};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::usize;

use rand::seq::IteratorRandom;
use rand::SeedableRng;
use rand::{prelude::*, Rng};
use rand_pcg::Pcg64;

type Point = (f32, f32);

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
        let points = file_to_points(path);
        let point_count = points.len();
        let adj_matrix = points_to_matrix(points);
        let sa = simulated_annealing(&adj_matrix, point_count);
        println!("SA: {:?}", sa.1);
    }
}

fn tabu_search() {
    todo!()
}

fn simulated_annealing(adj_matrix: &[Vec<usize>], mut temperature: usize) -> (Vec<usize>, usize) {
    let point_count = adj_matrix.len();
    let mut solution = get_random_permmutation(point_count);
    let mut current_weight = permutation_weight(&solution, adj_matrix);
    println!("SEED SOLUTION: {:?}", &current_weight);
    let mut rng = Pcg64::from_entropy();
    while temperature != 0 {
        for _epoch in 0..(1000) {
            let swap_idx = (0..point_count).choose_multiple(&mut Pcg64::from_entropy(), 2);
            let mut potential_solution = solution.clone();
            potential_solution.swap(swap_idx[0], swap_idx[1]);
            let potenital_weight = permutation_weight(&potential_solution, adj_matrix);
            if potenital_weight < current_weight {
                current_weight = potenital_weight;
                solution = potential_solution;
            } else if rng.gen_bool(E.powf((current_weight as f64 - potenital_weight as f64) / temperature as f64)) {
                current_weight = potenital_weight;
                solution = potential_solution;
            }
        }
        temperature -= 1;
    }
    (solution, current_weight)
}

fn get_random_permmutation(point_count: usize) -> Vec<usize> {
    let mut permutation: Vec<usize> = (0..point_count).collect();
    permutation.shuffle(&mut Pcg64::from_entropy());
    permutation
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn file_to_points<P>(filename: P) -> Vec<Point>
where
    P: AsRef<Path>,
{
    let mut points: Vec<Point> = Vec::new();
    if let Ok(lines) = read_lines(filename) {
        for line in lines.skip(8).flatten() {
            if line == "EOF" {
                break;
            }
            let tmp = line.split_whitespace().collect::<Vec<&str>>();
            points.push((tmp[1].parse().unwrap(), tmp[2].parse().unwrap()));
        }
    }
    points
}

fn points_to_matrix(points: Vec<(f32, f32)>) -> Vec<Vec<usize>> {
    let point_count = points.len();
    let mut adj_matrix: Vec<Vec<usize>> = vec![vec![0; point_count]; point_count];
    for i in 0..point_count {
        for j in i..point_count {
            if j != i {
                let p1 = points[i];
                let p2 = points[j];
                let dist =
                    (((p1.0 - p2.0).powi(2) + (p1.1 - p2.1).powi(2)).sqrt()).round() as usize;
                adj_matrix[i][j] = dist;
                adj_matrix[j][i] = dist;
            }
        }
    }
    adj_matrix
}

fn permutation_weight(permutation: &[usize], adj_matrix: &[Vec<usize>]) -> usize {
    let mut s: usize = 0;
    let mut prev = permutation[0];
    for cur in permutation.iter().skip(1) {
        s += adj_matrix[prev][*cur] as usize;
        prev = *cur;
    }
    s += *adj_matrix[0].last().unwrap() as usize;
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sa_test() {
        for path in ["test_data/1.tsp", "test_data/2.tsp", "test_data/3.tsp"] {
            let points = file_to_points(path);
            let point_count = points.len();
            let adj_matrix = points_to_matrix(points);
            let sa = simulated_annealing(&adj_matrix, point_count);
            println!("SA: {:?}", sa.1);
        }
    }
    #[test]
    fn ts_test() {
        for path in ["test_data/1.tsp", "test_data/2.tsp", "test_data/3.tsp"] {
            let points = file_to_points(path);
            let point_count = points.len();
            let adj_matrix = points_to_matrix(points);
            todo!()
            //tabu_search();
        }
    }
}