use std::collections::HashSet;
use std::f64::consts::E;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::usize;

use rand::seq::IteratorRandom;
use rand::SeedableRng;
use rand::{prelude::*, Rng};
use rand_pcg::Pcg64;

pub type Point = (f32, f32);

pub fn tabu_search(adj_matrix: &[Vec<usize>], tabu_capacity: usize) -> (Vec<usize>, usize) {
    let point_count = adj_matrix.len();
    let mut curr = get_random_permmutation(point_count);
    let mut curr_weight: usize = permutation_weight(&curr, adj_matrix);
    let mut best: Vec<usize> = curr.clone();
    let mut best_weight: usize = curr_weight;
    let mut tabu_list: HashSet<Vec<usize>> = HashSet::with_capacity(tabu_capacity); 
    tabu_list.insert(curr.clone());
    while tabu_list.len() < tabu_capacity {
        let mut neighborhood = get_neighborhood(&curr, adj_matrix, curr_weight);
        neighborhood.sort_by_key(|x| x.weight);
        for candidate in neighborhood {
            if !tabu_list.contains(&candidate.rep) {
                if best_weight > candidate.weight {
                    best = candidate.rep.clone();
                    best_weight = candidate.weight;
                }
                tabu_list.insert(candidate.rep.clone());
                curr = candidate.rep;
                curr_weight = candidate.weight;
                break;
            }
        }
    }
    (best, best_weight)
}

#[derive(Eq)]
struct Neighour {
    rep: Vec<usize>,
    weight: usize,
}
impl PartialEq for Neighour {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}
impl PartialOrd for Neighour {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Neighour {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.partial_cmp(other) {
            Some(x) => x,
            None => panic!("Something went wrong"),
        }
        
    }
}

fn get_neighborhood(
    permutation: &Vec<usize>,
    adj_matrix: &[Vec<usize>],
    weight: usize,
) -> Vec<Neighour> {
    let length = permutation.len();
    let mut neighborhood: Vec<Neighour> = Vec::new();
    for diff in 1..(length/2) {
        for j in diff..length {
            let mut rep = permutation.clone();
            rep[(j - diff)..=j].reverse();
            neighborhood.push(Neighour{rep, weight: invert_weight(permutation, adj_matrix, j - diff, j, weight)});
        }
    }
    neighborhood
}

fn invert_weight(
    permutation: &Vec<usize>,
    adj_matrix: &[Vec<usize>],
    i: usize,
    j: usize,
    weight: usize,
) -> usize {
    let last = permutation.len() - 1;
    let pre = i.checked_sub(1).unwrap_or(last);
    let post = (j + 1) % permutation.len();
    weight - adj_matrix[permutation[i]][permutation[pre]] - adj_matrix[permutation[j]][permutation[post]] + adj_matrix[permutation[j]][permutation[pre]] + adj_matrix[permutation[i]][permutation[post]]
}


pub fn simulated_annealing(adj_matrix: &[Vec<usize>], mut temperature: usize, epoch_count: usize) -> (Vec<usize>, usize) {
    let point_count = adj_matrix.len();
    let mut solution = get_random_permmutation(point_count);
    let mut current_weight = permutation_weight(&solution, adj_matrix);
    let mut rng = Pcg64::from_entropy();
    while temperature != 0 {
        for _epoch in 0..epoch_count {
            let swap_idx = (0..point_count).choose_multiple(&mut rng, 2);
            let mut potential_solution = solution.clone();
            potential_solution.swap(swap_idx[0], swap_idx[1]);
            let potenital_weight = permutation_weight(&potential_solution, adj_matrix);
            if potenital_weight < current_weight || rng.gen_bool(E.powf((current_weight as f64 - potenital_weight as f64) / temperature as f64)) {
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

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn file_to_points<P>(filename: P) -> Vec<Point>
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

pub fn points_to_matrix(points: Vec<(f32, f32)>) -> Vec<Vec<usize>> {
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

pub fn permutation_weight(permutation: &[usize], adj_matrix: &[Vec<usize>]) -> usize {
    let mut s: usize = 0;
    let mut prev = permutation[0];
    for cur in permutation.iter().skip(1) {
        s += adj_matrix[prev][*cur];
        prev = *cur;
    }
    s += *adj_matrix[0].last().unwrap();
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
            let _sa = simulated_annealing(&adj_matrix, point_count, 1000);
        }
    }
    #[test]
    fn ts_test() {
        for path in ["test_data/1.tsp", "test_data/2.tsp", "test_data/3.tsp"] {
            let points = file_to_points(path);
            let l = points.len();
            let adj_matrix = points_to_matrix(points);
            let _x = tabu_search(&adj_matrix, l);
        }
    }
}
