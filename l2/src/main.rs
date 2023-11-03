use std::fs::{self, File};
use std::io::{self, BufRead};
use std::path::Path;
use std::thread;

use rand::seq::IteratorRandom;
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
            permutation = local_search(permutation.clone(), &adj_matrix);
            //println!("{:?}", permutation_weight(&permutation, &adj_matrix));
        }

        let mut permutation: Vec<usize> = (0..point_count).collect();
        let mut handles = Vec::new();
        for _ in 0..point_count{
            permutation.shuffle(&mut rng);
            let permutation = permutation.clone();
            let adj_matrix =  adj_matrix.clone();
            //println!("{:?}", permutation_weight(&permutation, &adj_matrix));
            let handle = thread::spawn(move || {
                let p = local_search(permutation, &adj_matrix);
            });
            handles.push(handle);
            //println!("{:?}", permutation_weight(&permutation, &adj_matrix));
        }
        for handle in handles {
            handle.join().unwrap();
        }
        let mut handles = Vec::new();
        for _ in 0..point_count{
            permutation.shuffle(&mut rng);
            let permutation = permutation.clone();
            let adj_matrix =  adj_matrix.clone();
            //println!("{:?}", permutation_weight(&permutation, &adj_matrix));
            let handle = thread::spawn(move || {
                //println!("{:?}", permutation_weight(&permutation, &adj_matrix));
                let p = faster_local_search(permutation, &adj_matrix);
                //println!("{:?}", permutation_weight(&p, &adj_matrix));
            });
            handles.push(handle);
            //println!("{:?}", permutation_weight(&permutation, &adj_matrix));
        }
        for handle in handles {
            handle.join().unwrap();
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
    traversal
}

fn local_search( permutation: Vec<usize>, adj_matrix: &[Vec<u32>]) -> Vec<usize>{
    let mut curr_weight = permutation_weight(&permutation, adj_matrix);
    let mut curr = permutation.clone();
    loop {
        let neighborhood: Vec<Vec<usize>> = get_neighborhood(&curr);
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

fn faster_local_search( permutation: Vec<usize>, adj_matrix: &[Vec<u32>]) -> Vec<usize>{
    let mut curr_weight = permutation_weight(&permutation, adj_matrix);
    let mut curr = permutation.clone();
    let mut rng = Pcg64::from_entropy();
    loop {
        let neighborhood: Vec<Vec<usize>> = get_neighborhood(&curr);
        let neighborhood = neighborhood.iter().choose_multiple(&mut rng, permutation.len());
        let weights = neighborhood.iter().map(|x| permutation_weight(x, adj_matrix));
        let candidate = neighborhood.iter().zip(weights).min_by(|a, b| a.1.cmp(&b.1)).unwrap();
        if candidate.1 >= curr_weight {
            break;
        }
        curr = candidate.0.to_vec();
        curr_weight = candidate.1;
    }
    curr
}


fn get_neighborhood(permutation: &Vec<usize>) -> Vec<Vec<usize>> {
    let mut neighborhood: Vec<Vec<usize>> = Vec::new();
    let length = permutation.len();
    for diff in 1..length {
        for i in diff..length {
            neighborhood.push(invert(permutation.clone(), i-diff, i));
        }
    }
    neighborhood
}

fn invert(permutation: Vec<usize>, mut i: usize, mut j: usize) -> Vec<usize>{
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
    //s += *adj_matrix[0].last().unwrap() as u64;
    s
}
