use std::fs::{self, File};
use std::io::{self, BufRead};
use std::path::Path;

use rand::SeedableRng;
use rand_pcg::Pcg64;
use rand::prelude::*;

type Point = (f32, f32);

fn main() {
    let paths = fs::read_dir("test_data/").unwrap();
    for path in paths {
        let points = file_to_points(path.unwrap().path());
        let point_count = points.len();
        //println!("{:?}", points);
        let mut adj_matrix = vec![vec![u32::MAX; point_count]; point_count];
        for i in 0..point_count {
            for j in i..point_count {
                if j != i {
                    let p1 = points[i];
                    let p2 = points[j];
                    let dist =
                        (((p1.0 - p2.0).powi(2) + (p1.1 - p2.1).powi(2)).sqrt()).round() as u32;
                    adj_matrix[i][j] = dist;
                    adj_matrix[j][i] = dist;
                }
            }
        }
        let parent = prim(&adj_matrix, point_count);
        //println!("{:?}", &parent);
        let mst = parent_to_adj_list(&parent);
        //println!("{:?}", &mst);
        let mut rng = Pcg64::from_entropy();
        for _ in 0..((point_count as f32).sqrt() as usize) {
            let start = rng.gen_range(0..point_count);
            let mut permutation = dfs_from_point(&mst, start);
            //println!("{:?}", permutation_weight(&permutation, &adj_matrix));
            permutation = local_search(&mut permutation, &adj_matrix);
            //println!("{:?}", permutation_weight(&permutation, &adj_matrix));
        }

        let mut permutation: Vec<usize> = (0..point_count).collect();
        for _ in 0..point_count{
            permutation.shuffle(&mut rng);
            //println!("{:?}", permutation_weight(&permutation, &adj_matrix));
            permutation = local_search(&mut permutation, &adj_matrix);
            //println!("{:?}", permutation_weight(&permutation, &adj_matrix));
        }
    }
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

fn prim(adj_matrix: &[Vec<u32>], point_count: usize) -> Vec<usize> {
    let mut parent: Vec<usize> = vec![usize::MAX; point_count];
    let mut key: Vec<u32> = vec![u32::MAX; point_count];
    let mut mst_set: Vec<bool> = vec![false; point_count];

    key[0] = 0;

    for _ in 0..(point_count - 1) {
        let u = min_key(&key, &mst_set, point_count);
        mst_set[u] = true;
        for v in 0..point_count {
            if adj_matrix[u][v] != 0 && !mst_set[v] && adj_matrix[u][v] < key[v] {
                parent[v] = u;
                key[v] = adj_matrix[u][v];
            }
        }
    }

    parent
}

fn min_key(key: &[u32], mst_set: &[bool], point_count: usize) -> usize {
    let mut min = u32::MAX;
    let mut min_index = 0;
    for v in 0..point_count {
        if !mst_set[v] && key[v] < min {
            min = key[v];
            min_index = v;
        }
    }
    min_index
}

fn parent_to_adj_list(parent: &Vec<usize>) -> Vec<Vec<usize>> {
    let mut adj_list: Vec<Vec<usize>> = vec![Vec::new(); parent.len()];
    for (u, v) in parent.iter().enumerate().skip(1) {
        adj_list[u].push(*v);
        adj_list[*v].push(u);
    }
    adj_list
}

fn dfs_from_point(graph: &[Vec<usize>], start: usize) -> Vec<usize>{
    let mut visited: Vec<usize> = Vec::new();
    let mut traversal: Vec<usize> = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    visited.push(start);
    stack.push(start);
    while let Some(node) = stack.pop() {
        traversal.push(node);
        for j in &graph[node] {
            if !visited.contains(j) {
                visited.push(*j);
                stack.push(*j);
            }
        }
    }
    //println!("{:?}", traversal);
    traversal
}

fn local_search( permutation: &mut Vec<usize>, adj_matrix: &[Vec<u32>]) -> Vec<usize>{
    let length = permutation.len();
    let mut curr_weight = permutation_weight(permutation, adj_matrix);
    let mut curr = permutation.clone();
    loop {
        let mut neighborhood: Vec<Vec<usize>> = Vec::new();
        let mut i = 0;
        for j in (0..length).step_by(10).skip(1) {
            neighborhood.push(invert(&mut curr, i, j));
            i = j;
        }
        let weights = neighborhood.iter().map(|x| permutation_weight(x, adj_matrix));
        let candidate = neighborhood.iter().zip(weights).min_by(|a, b| a.1.cmp(&b.1)).unwrap();
        if candidate.1 >= curr_weight {
            break;
        }
        curr = candidate.0.clone();
        curr_weight = candidate.1;

    }
    curr
}

fn invert(permutation: &mut [usize], mut i: usize, mut j: usize) -> Vec<usize>{
    let mut permutation = permutation.to_owned();
    while i != j && j > i {
        permutation.swap(i, j);
        i += 1;
        j -= 1;
    }
    permutation
}

fn permutation_weight(permutation: &[usize], adj_matrix: &[Vec<u32>]) -> u64 {
    let mut s: u64 = 0;
    let mut prev = 0;
    for cur in permutation.iter().skip(1) {
        s += adj_matrix[prev][*cur] as u64;
        prev = *cur;
    }
    s
}
