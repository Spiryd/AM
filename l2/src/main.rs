use std::fs::{self, File};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use rand::seq::IteratorRandom;
use rand::SeedableRng;
use rand_pcg::Pcg64;
use rand::prelude::*;

type Point = (f32, f32);

fn main() {
    let mut weight_file = File::create("./ls.csv").unwrap();
    weight_file.write_all(b"map;mst_weight;dfs_steps;dfs_mean;dfs_min;random_steps;random_mean;random_min;mod_random_steps;mod_random_mean;mod_random_min\n").unwrap();
    let paths = fs::read_dir("test_data/").unwrap();
    for path in paths {
        let points = file_to_points(path.unwrap().path());
        let point_count = points.len();
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
        let mst = parent_to_adj_list(&parent);
        let mst_weight = mst_weight(&parent, &adj_matrix);
        let mut dfs_min = u64::MAX;
        let mut dfs_mean = 0_u64;
        let mut dfs_steps = 0_usize;
        let mut rng = Pcg64::from_entropy();
        for _ in 0..((point_count as f32).sqrt() as usize) {
            let start = rng.gen_range(0..point_count);
            let permutation = dfs_from_point(&mst, start);
            let (p, counter) = local_search(permutation.clone(), &adj_matrix);
            let w = permutation_weight(&p, &adj_matrix);
            dfs_mean += w;
            dfs_steps += counter;
            if dfs_min > w {
                dfs_min = w;
            }
        }
        let dfs_mean = dfs_mean as f64 / (point_count as f64).sqrt();
        let dfs_steps = dfs_steps as f64 / (point_count as f64).sqrt();

        let random_min  = Arc::new(Mutex::new(u64::MAX));
        let random_mean = Arc::new(Mutex::new(0_u64));
        let mut permutation: Vec<usize> = (0..point_count).collect();
        let random_steps = Arc::new(Mutex::new(0_usize));
        
        for _ in 0..(point_count/10){
            let mut handles = vec![];
            for _ in 0..10 {
                permutation.shuffle(&mut rng);
                let random_steps = Arc::clone(&random_steps);
                let random_mean = Arc::clone(&random_mean);
                let random_min = Arc::clone(&random_min);
                let permutation = permutation.clone();
                let adj_matrix =  adj_matrix.clone();
                let handle = thread::spawn(move || {
                    let (p, counter) = local_search(permutation, &adj_matrix);
                    let w = permutation_weight(&p, &adj_matrix);
                    let mut c = random_steps.lock().unwrap();
                    *c += counter;
                    let mut me = random_mean.lock().unwrap();
                    *me += w;
                    let mut mi = random_min.lock().unwrap();
                    if *mi > w {
                        *mi = w;
                    }
                });
                handles.push(handle);
            }
            for handle in handles {
                handle.join().unwrap();
            }
        }
        let random_mean = *random_mean.lock().unwrap() as f64 / point_count as f64;
        let random_steps = *random_steps.lock().unwrap() as f64 / point_count as f64;
        let random_min = *random_min.lock().unwrap();

        let mod_random_min  = Arc::new(Mutex::new(u64::MAX));
        let mod_random_mean = Arc::new(Mutex::new(0_u64));
        let mut permutation: Vec<usize> = (0..point_count).collect();
        let mod_random_steps = Arc::new(Mutex::new(0_usize));
        for _ in 0..(point_count/10){
            let mut handles = vec![];
            for _ in 0..10 {
                permutation.shuffle(&mut rng);
                let mod_random_steps = Arc::clone(&mod_random_steps);
                let mod_random_mean = Arc::clone(&mod_random_mean);
                let mod_random_min = Arc::clone(&mod_random_min);
                let permutation = permutation.clone();
                let adj_matrix =  adj_matrix.clone();
                let handle = thread::spawn(move || {
                    let (p, counter) = faster_local_search(permutation, &adj_matrix);
                    let w = permutation_weight(&p, &adj_matrix);
                    let mut c = mod_random_steps.lock().unwrap();
                    *c += counter;
                    let mut me = mod_random_mean.lock().unwrap();
                    *me += w;
                    let mut mi = mod_random_min.lock().unwrap();
                    if *mi > w {
                        *mi = w;
                    }
                });
                handles.push(handle);
            }
            for handle in handles {
                handle.join().unwrap();
            }
        }
        let mod_random_mean = *mod_random_mean.lock().unwrap() as f64 / point_count as f64;
        let mod_random_steps = *mod_random_steps.lock().unwrap() as f64 / point_count as f64;
        let mod_random_min = *mod_random_min.lock().unwrap();
        weight_file.write_all(format!("{point_count};{mst_weight};{dfs_steps};{dfs_mean};{dfs_min};{random_steps};{random_mean};{random_min};{mod_random_steps};{mod_random_mean};{mod_random_min}\n").as_bytes()).unwrap();
    }
}

fn mst_weight(tree: &Vec<usize>, adj_matrix: &[Vec<u32>]) -> u64 {
    let mut s: u64 = 0;
    for i in 1..(tree.len()) {
        s += adj_matrix[i][tree[i]] as u64
    }
    s
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

fn local_search( permutation: Vec<usize>, adj_matrix: &[Vec<u32>]) -> (Vec<usize>, usize){
    let mut curr_weight = permutation_weight(&permutation, adj_matrix);
    let mut curr = permutation.clone();
    let mut counter = 0;
    loop {
        counter += 1;
        let neighborhood: Vec<(usize, usize, u64)> = get_neighborhood(&curr, adj_matrix);
        let candidate = neighborhood.iter().min_by(|a, b| a.2.cmp(&b.2)).unwrap();
        if candidate.2 >= curr_weight {
            break;
        }
        curr = invert(curr, candidate.0, candidate.1);
        curr_weight = candidate.2;
    }
    (curr, counter)
}

fn faster_local_search(permutation: Vec<usize>, adj_matrix: &[Vec<u32>]) -> (Vec<usize>, usize) {
    let mut curr_weight = permutation_weight(&permutation, adj_matrix);
    let mut curr = permutation.clone();
    let mut counter = 0;
    loop {
        counter += 1;
        let neighborhood: Vec<(usize, usize, u64)> = get_faster_neighborhood(&curr, adj_matrix);
        let candidate = neighborhood.iter().min_by(|a, b| a.2.cmp(&b.2)).unwrap();
        if candidate.2 >= curr_weight {
            break;
        }
        curr = invert(curr, candidate.0, candidate.1);
        curr_weight = candidate.2;
    }
    (curr, counter)
}

fn get_neighborhood(permutation: &Vec<usize>, adj_matrix: &[Vec<u32>]) -> Vec<(usize, usize, u64)> {
    let mut neighborhood: Vec<(usize, usize, u64)> = Vec::new();
    let length = permutation.len();
    let weight  =  permutation_weight(permutation, adj_matrix);
    for diff in 1..(length-1) {
        for j in diff..length {
            neighborhood.push((j-diff, j, invert_weight(permutation, adj_matrix, j-diff, j, weight)));
        }
    }
    neighborhood
}

fn get_faster_neighborhood(permutation: &Vec<usize>, adj_matrix: &[Vec<u32>]) -> Vec<(usize, usize, u64)> {
    let mut neighborhood: Vec<(usize, usize, u64)> = Vec::new();
    let length = permutation.len();
    let weight  =  permutation_weight(permutation, adj_matrix);
    let mut candidates = Vec::new();
    let mut rng = Pcg64::from_entropy();
    for diff in 1..(length-1) {
        for j in diff..length {
            candidates.push((j-diff, j));
        }
    }
    for (i, j) in candidates.iter().choose_multiple(&mut rng, length) {
        neighborhood.push((*i, *j, invert_weight(permutation, adj_matrix, *i, *j, weight)));
    }
    neighborhood
}

fn invert_weight(permutation: &Vec<usize>, adj_matrix: &[Vec<u32>], i:usize, j:usize, weight: u64) -> u64 {
    let last = permutation.len() - 1;
    if i == 0 {
        let (sub1, add1) = (adj_matrix[permutation[last]][permutation[0]] as u64, adj_matrix[permutation[last]][permutation[j]] as u64);
        let (sub2, add2) = (adj_matrix[permutation[j]][permutation[j+1]] as u64, adj_matrix[permutation[last]][permutation[j]] as u64);
        weight - sub1 - sub2 + add1 + add2
    } else if j == last {
        let (sub1, add1) = (adj_matrix[permutation[i-1]][permutation[i]] as u64, adj_matrix[permutation[i]][permutation[0]] as u64);
        let (sub2, add2) = (adj_matrix[permutation[last]][permutation[0]] as u64, adj_matrix[permutation[0]][permutation[i]] as u64);
        weight - sub1 - sub2 + add1 + add2
    } else {
        let (sub1, add1) = (adj_matrix[permutation[i-1]][permutation[i]] as u64, adj_matrix[permutation[i]][permutation[j+1]] as u64);
        let (sub2, add2) = (adj_matrix[permutation[j]][permutation[j+1]] as u64, adj_matrix[permutation[i-1]][permutation[j]] as u64);
         weight - sub1 - sub2 + add1 + add2
    }
    
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
    s += *adj_matrix[0].last().unwrap() as u64;
    s
}
