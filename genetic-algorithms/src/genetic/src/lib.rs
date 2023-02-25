use std::fmt;
use clap::{Parser, ValueEnum};
use rand::{Rng, seq::{IteratorRandom, SliceRandom}, distributions::{Distribution, WeightedIndex}};
use log::{trace, debug, info};

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum SexMethod {
    One,
    Two,
    Uniform
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum SelectionMethod {
    Equal,
    Replacement,
    Remainder
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Problem {
    Knapsack
}

impl fmt::Display for SexMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SexMethod::One => { write!(f, "one") },
            SexMethod::Two => { write!(f, "two") },
            SexMethod::Uniform => { write!(f, "uniform") }
        }
    }
}

impl fmt::Display for SelectionMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SelectionMethod::Equal => { write!(f, "equal") },
            SelectionMethod::Replacement => { write!(f, "stochastic sampling with replacement") },
            SelectionMethod::Remainder => { write!(f, "remainder stochastic sampling") }
        }
    }
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Problem::Knapsack => { write!(f, "knapsack") },
        }
    }
}

/// Genetic algorithm to generate optimal solutions
#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
pub struct Args {
    /// The genetic alphabet
    #[arg(short, long, required = true)]
    pub alphabet: String,

    /// The file needed for whatever problem
    #[arg(long)]
    pub file: Option<String>,

    /// The initial population of genitors
    #[arg(short, long, num_args = 1..)]
    pub genitors: Vec<String>,

    /// Length of genotypes
    #[arg(short, long, required = true)]
    pub length: usize,

    /// The mutation rate
    #[arg(short, long, default_value_t = 0.01)]
    pub mutation_rate: f64,

    /// The number of genitors in each population
    #[arg(short, long, default_value_t = 50)]
    pub population: usize,

    /// The number of genotypes in each intermediate population
    #[arg(short, long, default_value_t = 100)]
    pub intermediate_population: usize,

    /// The problem for which to generate solutions
    #[arg(short = 'r', long, value_enum, default_value_t = Problem::Knapsack)]
    pub problem: Problem,

    /// The method used to produce subsequent generations from genitors
    #[arg(short = 'x', long, value_enum, default_value_t = SexMethod::Uniform)]
    pub sex_method: SexMethod,

    /// The method of selection used to produce genitors from a population
    #[arg(short, long, value_enum, default_value_t = SelectionMethod::Equal)]
    pub selection_method: SelectionMethod,

    /// The maximum number of generations 
    #[arg(short = 'e', long, default_value_t = 100)]
    pub max_generations: usize,

    /// Chance of skipping reproduction and adding genitors to next generation
    #[arg(short = 'k', long, default_value_t = 0.1)]
    pub skip: f64,

    /// Force mutation if one occurs
    #[arg(short = 'f', long, default_value_t = false)]
    pub force_mutation: bool,
}

pub fn mutate(mut child: (String, String), mutation_rate: f64, alphabet: Vec<char>, force: bool) -> (String, String) {
    let mut rng = rand::thread_rng();
    let old = child.clone();

    trace!("Force mutation: {force}");

    trace!("Testing Child 0 for mutations");
    for i in 0..child.0.len() {
        if rng.gen_bool(mutation_rate) {
            let mut temp = child.0.chars().collect::<Vec<char>>();

            trace!("Mutated gene {i} from: {}", temp[i]);
            if force {
                temp[i] = alphabet
                    .iter()
                    .map(|c| *c)
                    .filter(|c| *c != temp[i])
                    .collect::<Vec<char>>()
                    [rng.gen_range(0..alphabet.len() - 1)];
            } else {
                temp[i] = alphabet[rng.gen_range(0..alphabet.len())];
            }

            trace!("to: {}", temp[i]);
            child.0 = temp.iter().collect::<String>();
        }
    }

    trace!("Testing Child 1 for mutations");
    for i in 0..child.1.len() {
        if rng.gen_bool(mutation_rate) {
            let mut temp = child.1.chars().collect::<Vec<char>>();

            trace!("Mutated gene {i} from: {}", temp[i]);
            if force {
                temp[i] = alphabet
                    .iter()
                    .map(|c| *c)
                    .filter(|c| *c != temp[i])
                    .collect::<Vec<char>>()
                    [rng.gen_range(0..alphabet.len() - 1)];
            } else {
                temp[i] = alphabet[rng.gen_range(0..alphabet.len())];
            }

            trace!("to: {}", temp[i]);
            child.1 = temp.iter().collect::<String>();
        }
    }

    if old.0 != child.0 {
        debug!("Mutated child 0 from {} to {}", old.0, child.0);
    }

    if old.1 != child.1 {
        debug!("Mutated child 1 from {} to {}", old.1, child.1);
    }

    child
}

pub fn reproduce(
    parent: (String, String),
    sex_method: SexMethod,
    alphabet: Vec<char>,
    mutation_rate: f64,
    skip: f64,
    force: bool) -> (String, String)
{
    if parent.0.len() != parent.1.len() {
        panic!("Genitor lengths are not equal: {} != {}", parent.0, parent.1);
    }

    let mut rng = rand::thread_rng();
    let length = parent.0.len();

    if rng.gen_bool(skip) {
        debug!("Returning parents: {}, {}", parent.0, parent.1);
        return parent;
    } 

    // create an infinite iterator to switch between each parent
    let parents = vec![parent.0, parent.1];
    let mut p = parents.iter().cycle();

    let mut child = (String::new(), String::new());
    let mut num_points = match sex_method {
        SexMethod::One => { (1..length).choose_multiple(&mut rng, 1) },
        SexMethod::Two => { (1..length).choose_multiple(&mut rng, 2) },
        SexMethod::Uniform => {
            let num = rng.gen_range(3..length);
            (1..length).choose_multiple(&mut rng, num)
        }
    };

    num_points.sort();

    trace!("Crossover points: {:?}", num_points);

    let mut last = 0;

    for i in num_points {
        let temp = &p.next().unwrap();
        trace!("Taking {} alleles from parent {} for child 0: {}", i - last, temp, temp[last..i].to_string());
        child.0 += &temp[last..i];

        let temp = &p.next().unwrap();
        trace!("Taking {} alleles from parent {} for child 0: {}", i - last, temp, temp[last..i].to_string());
        child.1 += &temp[last..i];

        trace!("Child 0: {}", child.0);
        trace!("Child 1: {}", child.1);

        last = i;
        let _ = &p.next();
    }

    let temp = &p.next().unwrap();
    trace!("Taking {} alleles from parent {} for child 0: {}", length - last, temp, temp[last..length].to_string());
    child.0 += &temp[last..length];

    let temp = &p.next().unwrap();
    trace!("Taking {} alleles from parent {} for child 0: {}", length - last, temp, temp[last..length].to_string());
    child.1 += &temp[last..length];

    trace!("Child 0: {}", child.0);
    trace!("Child 1: {}", child.1);

    if mutation_rate > 0.0 {
        child = mutate(child, mutation_rate, alphabet.clone(), force);
    }

    debug!("Produced children: {}, {}", child.0, child.1);
    child
}

pub fn generate_genitors(
    fitness: impl Fn(&String) -> (usize, usize, f64),
    population: usize,
    length: usize,
    alphabet: Vec<char>
) -> Vec<String> {
    let mut v = Vec::<String>::new();
    let mut rng = rand::thread_rng();

    trace!("Creating genitors");
    while v.len() < population {
        let mut genitor = String::new();
        for _ in 0..length {
            genitor.push(alphabet[rng.gen_range(0..alphabet.len())]);
        }

        let (weight, value, fit) = fitness(&genitor);
        debug!("Created genitor: {genitor} (weight = {weight}, value = {value}, fitness = {fit})");
        v.push(genitor);
    }

    v
}

pub fn select_genitors(
    fitness: impl Fn(&String) -> (usize, usize, f64),
    genitors: Vec<String>,
    population: usize,
    selection_method: SelectionMethod
) -> Vec<String> {
    let mut rng = rand::thread_rng();
    let mut pool = Vec::<String>::new();
    let mut g: Vec<(String, f64)> = genitors
        .iter()
        .map(|genotype| (genotype.to_string(), fitness(&genotype).2))
        .collect();

    // prioritize the best performers
    g.sort_by(|(_, a), (_, b)| b.total_cmp(a));

    trace!("(Genitors, fitness) = {:?}", g);

    match selection_method {
        SelectionMethod::Equal => {
            pool = g.iter().map(|(genotype, _)| genotype.to_string()).collect();
        },
        SelectionMethod::Remainder => {
            let avg_fit = g.iter()
                .filter(|(_, f)| f > &0.0)
                .fold(0.0, |total, (_, f)| total + f) / g.len() as f64; 
            trace!("Average fitness = {avg_fit}");

            for (genotype, fitness) in &g {
                let mut f = fitness / avg_fit;

                while pool.len() < population && f > 0.0 {
                    if f > 1.0 {
                        trace!("Pushing {genotype} into the pool");
                        pool.push(genotype.to_string());
                        f =- 1.0;
                    } else {
                        if rng.gen_bool(f) {
                        trace!("Pushing {genotype} into the pool");
                            pool.push(genotype.to_string());
                        }

                        f = 0.0;
                    }
                }
            }

            let mut genitor = g.iter().cycle();

            while pool.len() < population {
                if let Some((genotype, _)) = genitor.next() {
                    pool.push(genotype.to_string());
                }
            }
        },
        SelectionMethod::Replacement => {
            let avg_fit = g.iter()
                .filter(|(_, f)| f > &0.0)
                .fold(0.0, |total, (_, f)| total + f) / g.len() as f64; 
            trace!("Average fitness = {avg_fit}");

            // add the genitors randomly proportionally to their fitness
            let dist = WeightedIndex::new(g.iter().map(|(_, fitness)| fitness.max(0.0))).expect("Unable to sample genitor population");

            while pool.len() < population {
                let temp = g[dist.sample(&mut rng)].0.to_string();
                trace!("Pushing {temp} into the pool");
                pool.push(temp);
            }
        }
    }

    trace!("pool = {:?}", pool);
    pool   
}

pub fn generate_generation(
    fitness: impl Fn(&String) -> (usize, usize, f64),
    genitors: Option<Vec<String>>,
    population: usize, 
    intermediate_population: usize,
    length: usize,
    alphabet: Vec<char>,
    selection_method: SelectionMethod,
    sex_method: SexMethod,
    mutation_rate: f64,
    skip: f64,
    force: bool,
    num_generation: usize
) -> Vec<String> {
    let mut rng = rand::thread_rng();
    let g: Vec<String> = match genitors {
        Some(mut g) => {
            if g.len() < population {
                g.append(&mut generate_genitors(&fitness, population - g.len(), length, alphabet.clone()));
            }

            g
        },
        None => {
            generate_genitors(&fitness, population, length, alphabet.clone())
        }
    };

    if !g.iter().fold(true, |good, genotype| good && genotype.len() == length ) {
        panic!("Genitor genotype is incorrect length!");
    }
    
    let pool = select_genitors(&fitness, g, intermediate_population, selection_method);

    let mut next = Vec::<String>::new();

    while next.len() < population {
        let temp: Vec<&String> = pool.choose_multiple(&mut rng, 2).collect();
        let parent = (temp[0].to_string(), temp[1].to_string());
        let child = reproduce(parent, sex_method, alphabet.clone(), mutation_rate, skip, force);

        trace!("{} made it into next generation", child.0);
        next.push(child.0);

        if next.len() > population {
            trace!("{} made it into next generation", child.1);
            next.push(child.1);
        }
    }

    next.sort_by(|a, b| fitness(&b).2.total_cmp(&fitness(&a).2));

    info!("Generation {num_generation}");
    info!("--------------------------");

    for g in &next {
        let (w, v, f) = fitness(&g);
        info!("{g}: {w}, {v}, {f}");
    }

    info!("--------------------------");
    info!("");
    next
}
