use std::fs::{self, File};
use std::io::{self, BufRead};
use std::path::Path;

type Point = (f32, f32);

fn main() {
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
        let parent = prim(&adj_matrix, point_count);
        //println!("{:?}", &parent);
        let mst = parent_to_adj_list(&parent);
        //println!("{:?}", &mst);
    }
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
    for (u,v) in parent.iter().enumerate().skip(1) {
        adj_list[u].push(*v);
        adj_list[*v].push(u);
    }
    adj_list
}
