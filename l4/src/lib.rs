use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use rand::prelude::*;
use rand_pcg::Pcg64Mcg;
use rayon::prelude::*;

pub type Point = (f32, f32);

#[derive(Clone, Debug)]
struct Individual {
    chromosome: Vec<usize>,
    fitness: usize,
}

pub struct Evolution {
    islands: Vec<Vec<Individual>>,
    adj_matrix: Vec<Vec<usize>>,
    rng: Pcg64Mcg,
}

impl Evolution {
    pub fn new(island_count: usize, point_count: usize, adj_matrix: Vec<Vec<usize>>) -> Self {
        let mut islands: Vec<Vec<Individual>> = Vec::with_capacity(island_count);
        let mut rng = Pcg64Mcg::from_entropy();
        let mst = gen_mst(&adj_matrix, point_count);
        let fitness_mst = permutation_weight(&mst, &adj_matrix);
        let mst_individual = Individual { chromosome: mst, fitness: fitness_mst };
        let mut chromosome: Vec<usize> = (0..point_count).collect();
        for _ in 0..island_count {
            let mut island: Vec<Individual> = Vec::with_capacity(100);
            island.push(mst_individual.clone());
            for _ in 0..99 {
                chromosome.shuffle(&mut rng);
                let fitness = permutation_weight(&chromosome, &adj_matrix);
                island.push(Individual { chromosome: chromosome.clone(), fitness });
            }
            islands.push(island);
        }
        Evolution {
            islands,
            adj_matrix,
            rng,
        }
    }
    pub fn run(&mut self, pmx: bool) {
        let mut generation = 0;
        let mut no_improvement = 0;
        let mut curr_best = self.extract_best();
        loop {
            self.selection();
            if generation % 100 == 0 {
                self.migration();
                //println!("Generation: {}", generation);
            }
            self.reproduction(pmx);
            generation += 1;
            let new_best = self.extract_best();
            if new_best.1 < curr_best.1 {
                curr_best = new_best;
                no_improvement = 0;
            } else {
                no_improvement += 1;
            }
            if generation == 10000 || no_improvement == 250 {
                break;
            }
        }
    }
    pub fn extract_best(&self) -> (Vec<usize>, usize) {
        let mut best = self.islands[0][0].clone();
        for island in &self.islands {
            for individual in island {
                if individual.fitness < best.fitness {
                    best = individual.clone();
                }
            }
        }
        (best.chromosome, best.fitness)
    }
    fn reproduction(&mut self, pmx: bool) {
        self.islands.par_iter_mut().for_each(|op| {
            let mut new_population: Vec<Individual> = Vec::with_capacity(100);
            for _ in 0..100 {
                let parents = op.choose_multiple(&mut thread_rng(), 2).cloned().collect::<Vec<_>>();
                let (mut child1, mut child2): (Vec<usize>, Vec<usize>) = if pmx {
                    Self::pmx_crossover(&mut thread_rng(), &parents[0].chromosome, &parents[0].chromosome)
                } else {
                    Self::cx_crossover(&parents[0].chromosome, &parents[1].chromosome)
                };
                let fitness1 = permutation_weight(&child1, &self.adj_matrix);
                let fitness2 = permutation_weight(&child2, &self.adj_matrix);
                if thread_rng().gen_bool(0.1) {
                    Self::mutation(&mut thread_rng(), &mut child1);
                }
                if thread_rng().gen_bool(0.1) {
                    Self::mutation(&mut thread_rng(), &mut child2);
                }
                if new_population.len() == 100 {
                    break;
                }
                new_population.push(Individual {
                    chromosome: child1,
                    fitness: fitness1,
                });
                new_population.push(Individual {
                    chromosome: child2,
                    fitness: fitness2,
                });
            }
            *op = new_population;
        });
    }
    fn selection(&mut self) {
        self.islands.par_iter_mut().for_each(|island| {
            let mut new_population: Vec<Individual> = Vec::with_capacity(100);
            for _ in 0..100 {
                let mut tournament: Vec<Individual> = Vec::with_capacity(5);
                for _ in 0..5 {
                    tournament.push(island.choose(&mut thread_rng()).unwrap().clone());
                }
                tournament.sort_by_key(|x| x.fitness);
                new_population.push(tournament[0].clone());
            }
            *island = new_population;
        });
    }
    fn migration(&mut self) {
        let island_count = self.islands.len();
        for _ in 0..island_count {
            let source_island = self.rng.gen_range(0..island_count);
            let target_island = self.rng.gen_range(0..island_count);
            if source_island != target_island && !self.islands[source_island].is_empty() {
                let individual_index = self.rng.gen_range(0..self.islands[source_island].len());
                let individual = self.islands[source_island].remove(individual_index);
                self.islands[target_island].push(individual);
            }
        }
    }
    fn pmx_crossover(rng: &mut impl Rng, parent1: &Vec<usize>, parent2: &Vec<usize>) -> (Vec<usize>, Vec<usize>) {
        let len = parent1.len();
        let crossover_point1 = rng.gen_range(0..len);
        let crossover_point2 = rng.gen_range(0..len);
        let (start, end) = if crossover_point1 < crossover_point2 {
            (crossover_point1, crossover_point2)
        } else {
            (crossover_point2, crossover_point1)
        };
    
        let mut child1 = parent1.clone();
        let mut child2 = parent2.clone();
    
        child1[start..end].clone_from_slice(&parent2[start..end]);
        child2[start..end].clone_from_slice(&parent1[start..end]);
    
        for i in 0..len {
            if i < start || i > end {
                while child1[start..end].contains(&parent1[i]) {
                    let index = child1[start..end].iter().position(|&x| x == parent1[i]).unwrap();
                    child1[i] = parent2[start + index];
                }
                while child2[start..end].contains(&parent2[i]) {
                    let index = child2[start..end].iter().position(|&x| x == parent2[i]).unwrap();
                    child2[i] = parent1[start + index];
                }
            }
        }
    
        (child1, child2)
    }

    fn cx_crossover(parent1: &Vec<usize>, parent2: &Vec<usize>) -> (Vec<usize>, Vec<usize>) {
        let len = parent1.len();
        let mut child1 = vec![None; len];
        let mut child2 = vec![None; len];
    
        let mut index = 0;
        let mut cycle = 0;
        while cycle < len {
            while child1[index].is_none() {
                child1[index] = Some(parent1[index]);
                child2[index] = Some(parent2[index]);
                index = parent2.iter().position(|&x| x == parent1[index]).unwrap();
            }
            cycle += 1;
            index = (index + 1) % len;
        }
    
        for i in 0..len {
            if child1[i].is_none() {
                child1[i] = Some(parent2[i]);
                child2[i] = Some(parent1[i]);
            }
        }
    
        (child1.into_iter().map(|x| x.unwrap()).collect(), child2.into_iter().map(|x| x.unwrap()).collect())
    }
    fn mutation(rng: &mut impl Rng, child: &mut Vec<usize>) {
        if child.len() < 2 {
            return;
        }
        let start = rng.gen_range(0..child.len() - 1);
        let end = rng.gen_range(start + 1..child.len());
        child[start..end].reverse();
    }
}

pub fn permutation_weight(permutation: &[usize], adj_matrix: &[Vec<usize>]) -> usize {
    let result = permutation.par_chunks(1000)
        .map(|permutation_chunk| {
            let mut s: usize = 0;
            let mut prev = permutation_chunk[0];
            for cur in permutation_chunk.iter().skip(1) {
                s += adj_matrix[prev][*cur];
                prev = *cur;
            }
            s += *adj_matrix[0].last().unwrap();
            s
        })
        .sum();

    result
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

fn gen_mst(adj_matrix: &[Vec<usize>], point_count: usize) -> Vec<usize> {
    let parent = prim(&adj_matrix, point_count);
    let mst = parent_to_adj_list(&parent);
    dfs(&mst)
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
    traversal
}

fn prim(adj_matrix: &[Vec<usize>], point_count: usize) -> Vec<usize> {
    let mut parent: Vec<usize> = vec![usize::MAX; point_count];
    let mut key: Vec<usize> = vec![usize::MAX; point_count];
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

fn min_key(key: &[usize], mst_set: &[bool], point_count: usize) -> usize {
    let mut min = usize::MAX;
    let mut min_index = 0;
    for v in 0..point_count {
        if !mst_set[v] && key[v] < min {
            min = key[v];
            min_index = v;
        }
    }
    min_index
} 

