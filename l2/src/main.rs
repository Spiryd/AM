use std::fs::{self, File};
use std::io::{self, BufRead, Write};
use std::path::Path;

use rand::prelude::*;
use rand::seq::IteratorRandom;
use rand::SeedableRng;
use rand_pcg::Pcg64;

type Point = (f32, f32);

fn main() {
    let mut weight_file = File::create("./ls.csv").unwrap();
    weight_file.write_all(b"map;mst_weight;dfs_steps;dfs_mean;dfs_min;random_steps;random_mean;random_min;mod_random_steps;mod_random_mean;mod_random_min\n").unwrap();
    let paths = fs::read_dir("test_data/").unwrap();
    //for path in paths {
    for path in ["test_data/c.tsp", "test_data/d.tsp", "test_data/e.tsp", "test_data/f.tsp"] {
        //let points = file_to_points(path.unwrap().path());
        let points = file_to_points(path);
        let point_count = points.len();
        let adj_matrix = points_to_matrix(points);
        let parent = prim(&adj_matrix, point_count);
        let mst = parent_to_adj_list(&parent);
        let mst_weight = mst_weight(&parent, &adj_matrix);

        let mut dfs_min = u64::MAX;
        let mut dfs_mean = 0_u64;
        let mut dfs_steps = 0_usize;
        let mut rng = Pcg64::from_entropy();
        //for _ in 0..((point_count as f32).sqrt() as usize) {
        for _ in 0..100 {
            let start = rng.gen_range(0..point_count);
            let permutation = dfs_from_point(&mst, start);
            let (_p, counter, w) = local_search(permutation.clone(), &adj_matrix);
            dfs_mean += w;
            dfs_steps += counter;
            if dfs_min > w {
                dfs_min = w;
            }
        }
        let dfs_mean = dfs_mean as f64 / (point_count as f64).sqrt();
        let dfs_steps = dfs_steps as f64 / (point_count as f64).sqrt();
        
        let mut random_min = u64::MAX;
        let mut random_mean = 0_u64;
        let mut permutation: Vec<usize> = (0..point_count).collect();
        let mut random_steps = 0_usize;
        //for _ in 0..point_count {
        for _ in 0..100 {
            permutation.shuffle(&mut rng);
            //println!("local search start");
            let (_p, counter, w) = local_search(permutation.clone(), &adj_matrix);
            //println!("local search end");
            random_steps += counter;
            random_mean += w;
            if random_min > w {
                random_min = w;
            }
        }
        println!("random end");
        let random_mean = random_mean as f64 / point_count as f64;
        let random_steps = random_steps as f64 / point_count as f64;
        
        let mut mod_random_min = u64::MAX;
        let mut mod_random_mean = 0_u64;
        let mut permutation: Vec<usize> = (0..point_count).collect();
        let mut mod_random_steps = 0_usize;
        //for _ in 0..point_count {
        for _ in 0..100 {
            permutation.shuffle(&mut rng);
            //println!("local search start");
            let (_p, counter, w) = faster_local_search(permutation.clone(), &adj_matrix);
            //println!("local search end");
            mod_random_steps += counter;
            mod_random_mean += w;
            if mod_random_min > w {
                mod_random_min = w;
            }
        }
        let mod_random_mean = mod_random_mean as f64 / point_count as f64;
        let mod_random_steps = mod_random_steps as f64 / point_count as f64;
        
        weight_file.write_all(format!("{point_count};{mst_weight};{dfs_steps};{dfs_mean};{dfs_min};{random_steps};{random_mean};{random_min};{mod_random_steps};{mod_random_mean};{mod_random_min}\n").as_bytes()).unwrap();
        //weight_file.write_all(format!("{point_count};{mst_weight};{dfs_steps};{dfs_mean};{dfs_min};{mod_random_steps};{mod_random_mean};{mod_random_min}\n").as_bytes()).unwrap();
        //weight_file.write_all(format!("{point_count};{mst_weight};{dfs_steps};{dfs_mean};{dfs_min}\n").as_bytes()).unwrap();

    }
}

fn mst_weight(tree: &Vec<usize>, adj_matrix: &[Vec<u64>]) -> u64 {
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

fn prim(adj_matrix: &[Vec<u64>], point_count: usize) -> Vec<usize> {
    let mut parent: Vec<usize> = vec![usize::MAX; point_count];
    let mut key: Vec<u64> = vec![u64::MAX; point_count];
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

fn min_key(key: &[u64], mst_set: &[bool], point_count: usize) -> usize {
    let mut min = u64::MAX;
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

fn dfs_from_point(graph: &[Vec<usize>], start: usize) -> Vec<usize> {
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

fn local_search(permutation: Vec<usize>, adj_matrix: &[Vec<u64>]) -> (Vec<usize>, usize, u64) {
    let mut curr_weight = permutation_weight(&permutation, adj_matrix);
    let mut curr = permutation.clone();
    let mut counter = 0;
    loop {
        counter += 1;
        let neighborhood: Vec<(usize, usize, u64)> = get_neighborhood(&curr, adj_matrix, curr_weight
        );
        let candidate = neighborhood.iter().min_by_key(|a| a.2).unwrap();
        //let candidate = get_candidate(&permutation, adj_matrix, curr_weight);
        //println!("exp:{:?} got:{:?}", permutation_weight(&invert(curr.clone(), candidate.0, candidate.1), adj_matrix), candidate.2);
        if candidate.2 >= curr_weight {
            break;
        }
        curr[candidate.0..=candidate.1].reverse();
        //println!("curr: {:?}", curr);
        curr_weight = candidate.2;
        //println!("w: {:?}", curr_weight)
    }
    //println!("w: {:?}", curr_weight);
    (curr, counter, curr_weight)
}

fn faster_local_search(permutation: Vec<usize>, adj_matrix: &[Vec<u64>]) -> (Vec<usize>, usize, u64) {
    let mut curr_weight = permutation_weight(&permutation, adj_matrix);
    let mut curr = permutation.clone();
    let mut counter = 0;
    loop {
        counter += 1;
        let neighborhood: Vec<(usize, usize, u64)> = get_faster_neighborhood(&curr, adj_matrix);
        let candidate = neighborhood.iter().min_by_key(|a| a.2).unwrap();
        if candidate.2 >= curr_weight {
            break;
        }
        curr[candidate.0..=candidate.1].reverse();
        curr_weight = candidate.2;
    }
    (curr, counter, curr_weight)
}

fn get_neighborhood(
    permutation: &Vec<usize>,
    adj_matrix: &[Vec<u64>],
    weight: u64,
) -> Vec<(usize, usize, u64)> {
    let mut neighborhood: Vec<(usize, usize, u64)> = Vec::new();
    let length = permutation.len();
    for diff in 1..(length/2) {
        for j in diff..length {
            neighborhood.push((
                j - diff,
                j,
                invert_weight(permutation, adj_matrix, j - diff, j, weight),
            ));
        }
    }
    neighborhood
}

fn get_faster_neighborhood(
    permutation: &Vec<usize>,
    adj_matrix: &[Vec<u64>],
) -> Vec<(usize, usize, u64)> {
    let mut neighborhood: Vec<(usize, usize, u64)> = Vec::new();
    let length = permutation.len();
    let weight = permutation_weight(permutation, adj_matrix);
    let mut candidates = Vec::new();
    let mut rng = Pcg64::from_entropy();
    for diff in 1..(length/2) {
        for j in diff..length {
            candidates.push((j - diff, j));
        }
    }
    for (i, j) in candidates.iter().choose_multiple(&mut rng, length) {
        neighborhood.push((
            *i,
            *j,
            invert_weight(permutation, adj_matrix, *i, *j, weight),
        ));
    }
    neighborhood
}

fn invert_weight(
    permutation: &Vec<usize>,
    adj_matrix: &[Vec<u64>],
    i: usize,
    j: usize,
    weight: u64,
) -> u64 {
    let last = permutation.len() - 1;
    let pre = i.checked_sub(1).unwrap_or(last);
    let post = (j + 1) % permutation.len();
    weight - adj_matrix[permutation[i]][permutation[pre]] - adj_matrix[permutation[j]][permutation[post]] + adj_matrix[permutation[j]][permutation[pre]] + adj_matrix[permutation[i]][permutation[post]]
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
    //println!("{:?}", adj_matrix);
    adj_matrix
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_task_test() {
        for path in [
            "test_data/1.tsp",
            "test_data/2.tsp",
            "test_data/3.tsp",
        ] {
            let points = file_to_points(path);
            let point_count = points.len();
            let adj_matrix = points_to_matrix(points);
            let parent = prim(&adj_matrix, point_count);
            let mst = parent_to_adj_list(&parent);
            let mut dfs_min = u64::MAX;
            let mut dfs_mean = 0_u64;
            let mut dfs_steps = 0_usize;
            let mut rng = Pcg64::from_entropy();
            for _ in 0..((point_count as f32).sqrt() as usize) {
                let start = rng.gen_range(0..point_count);
                let permutation = dfs_from_point(&mst, start);
                let (_p, counter, w) = local_search(permutation.clone(), &adj_matrix);
                dfs_mean += w;
                dfs_steps += counter;
                if dfs_min > w {
                    dfs_min = w;
                }
                //println!("w: {:?}", w);
            }
            let dfs_mean = dfs_mean as f64 / (point_count as f64).sqrt();
            let dfs_steps = dfs_steps as f64 / (point_count as f64).sqrt();
            println!("{dfs_min}, {dfs_mean}, {dfs_steps}");
        }
    }
    #[test]
    fn second_task_test() {
        for path in [
            "test_data/1.tsp",
            "test_data/2.tsp",
            "test_data/3.tsp",
        ] {
            let points = file_to_points(path);
            let point_count = points.len();
            let adj_matrix = points_to_matrix(points);
            let mut random_min = u64::MAX;
            let mut random_mean = 0_u64;
            let mut permutation: Vec<usize> = (0..point_count).collect();
            let mut random_steps = 0_usize;
            let mut rng = Pcg64::from_entropy();
            for _ in 0..point_count {
                permutation.shuffle(&mut rng);
                //println!("local search start");
                let (_p, counter, w) = local_search(permutation.clone(), &adj_matrix);
                //println!("local search end");
                random_steps += counter;
                random_mean += w;
                if random_min > w {
                    random_min = w;
                }
            }
            let random_mean = random_mean as f64 / point_count as f64;
            let random_steps = random_steps as f64 / point_count as f64;
            println!("{random_min}, {random_mean}, {random_steps}");
        }
    }
    #[test]
    fn third_task_test() {
        for path in [
            "test_data/1.tsp",
            "test_data/2.tsp",
            "test_data/3.tsp",
        ] {
            let points = file_to_points(path);
            let point_count = points.len();
            let adj_matrix = points_to_matrix(points);
            let mut random_min = u64::MAX;
            let mut random_mean = 0_u64;
            let mut permutation: Vec<usize> = (0..point_count).collect();
            let mut random_steps = 0_usize;
            let mut rng = Pcg64::from_entropy();
            for _ in 0..point_count {
                permutation.shuffle(&mut rng);
                //println!("local search start");
                let (_p, counter, w) = faster_local_search(permutation.clone(), &adj_matrix);
                //println!("local search end");
                random_steps += counter;
                random_mean += w;
                if random_min > w {
                    random_min = w;
                }
            }
            let random_mean = random_mean as f64 / point_count as f64;
            let random_steps = random_steps as f64 / point_count as f64;
            println!("{random_min}, {random_mean}, {random_steps}");
        }
    }
}
