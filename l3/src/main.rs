use std::fs::{self, File};
use std::io::{self, BufRead, Write};
use std::path::Path;

use rand::prelude::*;
use rand::seq::IteratorRandom;
use rand::SeedableRng;
use rand_pcg::Pcg64;

type Point = (f32, f32);

fn main() {
    for path in ["test_data/b.tsp", "test_data/c.tsp", "test_data/d.tsp", "test_data/e.tsp", "test_data/f.tsp"] {
        let points = file_to_points(path);
        let point_count = points.len();
        let adj_matrix = points_to_matrix(points);
    }
}


fn tabu_search() {
    todo!()
}

fn simulated_annealing () {
    todo!()
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

fn points_to_matrix(points: Vec<(f32, f32)>) -> Vec<Vec<u64>> {
    let point_count = points.len();
    let mut adj_matrix: Vec<Vec<u64>> = vec![vec![0; point_count]; point_count];
    for i in 0..point_count {
        for j in i..point_count {
            if j != i {
                let p1 = points[i];
                let p2 = points[j];
                let dist = (((p1.0 - p2.0).powi(2) + (p1.1 - p2.1).powi(2)).sqrt()).round() as u64;
                adj_matrix[i][j] = dist;
                adj_matrix[j][i] = dist;
            }
        }
    }
    adj_matrix
}

fn permutation_weight(permutation: &[usize], adj_matrix: &[Vec<u64>]) -> u64 {
    let mut s: u64 = 0;
    let mut prev = permutation[0];
    for cur in permutation.iter().skip(1) {
        s += adj_matrix[prev][*cur] as u64;
        prev = *cur;
    }
    s += *adj_matrix[0].last().unwrap() as u64;
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sa_test() {
        for path in [
            "test_data/1.tsp",
            "test_data/2.tsp",
            "test_data/3.tsp",
        ] {
            let points = file_to_points(path);
            let point_count = points.len();
            let adj_matrix = points_to_matrix(points);
            let mut permutation: Vec<usize> = (0..point_count).collect();
            simulated_annealing();
        }
    }
    #[test]
    fn ts_test() {
        for path in [
            "test_data/1.tsp",
            "test_data/2.tsp",
            "test_data/3.tsp",
        ] {
            let points = file_to_points(path);
            let point_count = points.len();
            let adj_matrix = points_to_matrix(points);
            let mut permutation: Vec<usize> = (0..point_count).collect();
            tabu_search();
        }
    }
}

