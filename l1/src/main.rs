use std::fs::{self, File};
use std::io::{self, BufRead, Write};
use std::path::Path;

use rand::prelude::*;
use rand_pcg::Pcg64;
use serde_pickle::SerOptions;

type Point = (f32, f32);

fn main() {
    let mut weight_file = File::create("./weights.csv").unwrap();
    weight_file.write_all(b"map;mst_weight;dfs_weight;a_avg;b_avg;random_min\n").unwrap();

    let paths = fs::read_dir("test_data/").unwrap();
    for path in paths {
        let points = file_to_points(path.unwrap().path());
        let point_count = points.len();
        //println!("{:?}", points);
        let mut adj_matrix = vec![vec![u32::MAX; point_count]; point_count];
        for i in 0..point_count {
            for j in i..point_count  {
                if j != i {
                    let p1 = points[i];
                    let p2 = points[j];
                    let dist = (((p1.0 - p2.0).powi(2) + (p1.1 - p2.1).powi(2)).sqrt()).round() as u32;
                    adj_matrix[i][j] = dist;
                    adj_matrix[j][i] = dist;
                }
            }
        }
        //println!("{:?}", adj_matrix);
        let parent = prim(&adj_matrix, point_count);
        //println!("{:?}", &mst);
        let mst_weight = weight(&parent, &adj_matrix);
        //println!("{:?}", &mst_weight);
        let mst = parent_to_adj_list(&parent);
        //println!("{:?}", &mst);
        let traversal = dfs(&mst);
        //println!("{:?}", &traversal);

        let mut dfs_file = File::create(format!("./routes/dfs_{point_count}_route.bin")).unwrap();
        serde_pickle::to_writer(&mut dfs_file, &traversal.iter().map(|x| points[*x]).collect::<Vec<Point>>(), SerOptions::new()).unwrap();

        let dfs_weight = weight_traversal(&traversal, &adj_matrix);
        //println!("{:?}", &dfs_weight);

        let mut point_ids: Vec<usize> = (0..point_count).collect();
        let mut rng: rand_pcg::Lcg128Xsl64 = Pcg64::from_entropy();

        let mut weights: Vec<u64> = Vec::new();
        let mut permutations: Vec<Vec<usize>> = Vec::new();
        for _ in 0..1000 {
            point_ids.shuffle(&mut rng);
            weights.push(weight_traversal(&point_ids, &adj_matrix));
            permutations.push(point_ids.clone());
        }

        let mut a_avg: f64 = 0.;
        for perms in weights.chunks(10)  {
            a_avg += *perms.iter().min().unwrap() as f64;
        }
        a_avg /= 100.;

        let mut b_avg: f64 = 0.;
        for perms in weights.chunks(50)  {
            b_avg += *perms.iter().min().unwrap() as f64;
        }
        b_avg /= 20.;
        let min = weights.iter().enumerate().min_by_key(|&(_, item)| item).unwrap();
        
        let mut rand_file = File::create(format!("./routes/rand_{point_count}_route.bin")).unwrap();
        serde_pickle::to_writer(&mut rand_file, &permutations[min.0].iter().map(|x| points[*x]).collect::<Vec<Point>>(), SerOptions::new()).unwrap();
        
        weight_file.write_all(format!("{point_count};{mst_weight};{dfs_weight};{a_avg};{b_avg};{}\n", min.1).as_bytes()).unwrap();
    }
}

fn weight(tree: &Vec<usize>, adj_matrix: &[Vec<u32>]) -> u64 {
    let mut s: u64 = 0;
    for i in 1..(tree.len()) {
        s += adj_matrix[i][tree[i]] as u64
    }
    s
}

fn weight_traversal(traversal: &[usize], adj_matrix: &[Vec<u32>]) -> u64 {
    let mut s: u64 = 0;
    let mut prev = 0;
    for cur in traversal.iter().skip(1) {
        s += adj_matrix[prev][*cur] as u64;
        prev = *cur;
    }
    s
}

fn parent_to_adj_list(parent: &Vec<usize>) -> Vec<Vec<usize>> {
    let mut adj_list: Vec<Vec<usize>> = vec![Vec::new(); parent.len()];
    for (u,v) in parent.iter().enumerate().skip(1) {
        adj_list[u].push(*v);
        adj_list[*v].push(u);
    }
    adj_list
}

fn dfs(graph: &Vec<Vec<usize>>) -> Vec<usize>{
    let mut visited: Vec<usize> = Vec::new();
    let mut traversal: Vec<usize> = Vec::new();
    for i in 0..graph.len() {
        if !visited.contains(&i) {
            let mut stack: Vec<usize> = Vec::new();
            visited.push(i);
            stack.push(i);
            while let Some(node) = stack.pop() {
                traversal.push(node);
                for j in &graph[node] {
                    if !visited.contains(j) {
                        visited.push(*j);
                        stack.push(*j);
                    }
                }
            }
        }
    }
    //println!("{:?}", traversal);
    traversal
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

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path> {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn file_to_points<P>(filename: P) -> Vec<Point> 
where P: AsRef<Path> {
    let mut points: Vec<Point> = Vec::new();
    if let Ok(lines) = read_lines(filename) {
        for line in lines.skip(8).flatten() {
            if line == "EOF"{
                break;
            }
            let tmp = line.split_whitespace().collect::<Vec<&str>>();
            points.push((tmp[1].parse().unwrap(), tmp[2].parse().unwrap()));
        
        }
    }
    points
}
