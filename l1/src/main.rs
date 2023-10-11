use std::fs::{self, File};
use std::io::{self, BufRead};
use std::path::Path;

type Point = (u16, u16);

fn main() {
    let paths = fs::read_dir("data/").unwrap();
    for path in paths {
        let points = file_to_points(path.unwrap().path());
        println!("{:?}", points)
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
        for line in lines.skip(8) {
            if let Ok(line) = line {
                if line == "EOF"{
                    break;
                }
                let tmp = line.split_whitespace().collect::<Vec<&str>>();
                points.push((tmp[1].parse().unwrap(), tmp[2].parse().unwrap()));
            }
        }
    }
    points
}
