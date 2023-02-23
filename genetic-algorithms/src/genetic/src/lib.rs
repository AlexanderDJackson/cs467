use std::fmt;
use clap::{Parser, ValueEnum};
use rand::{Rng, seq::IteratorRandom, distributions::{Distribution, WeightedIndex}};
use log::{trace, debug, info, warn, error, LevelFilter};

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
    #[arg(short, long)]
    pub file: String,

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
    #[arg(short, long, default_value_t = 10)]
    pub population: usize,

    /// The problem for which to generate solutions
    #[arg(short = 'r', long, value_enum, default_value_t = Problem::Knapsack)]
    pub problem: Problem,

    /// The method used to produce subsequent generations from genitors
    #[arg(short = 'x', long, value_enum, default_value_t = SexMethod::Uniform)]
    pub sex_method: SexMethod,

    /// The method of selection used to produce genitors from a population
    #[arg(short, long, value_enum, default_value_t = SelectionMethod::Equal)]
    pub selection_method: SelectionMethod,
}

pub fn mutate(mut child: (String, String), mutation_rate: f64, alphabet: Vec<char>, force: bool) -> (String, String) {
    let mut rng = rand::thread_rng();

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

    debug!("Mutated child 0: {}", child.0);
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

    debug!("Mutated child 1: {}", child.1);
    trace!("Mutated children: {}, {}", child.0, child.1);

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
    let mut rng = rand::thread_rng();

    if rng.gen_bool(skip) {
        debug!("Returning parents: {}, {}", parent.0, parent.1);
        return parent;
    }

    let mut child = (String::new(), String::new());
    let num_points = match sex_method {
        SexMethod::One => { (0..parent.0.len()).choose_multiple(&mut rng, 1) },
        SexMethod::Two => { (0..parent.0.len()).choose_multiple(&mut rng, 2) },
        SexMethod::Uniform => {
            let num = rng.gen_range(0..parent.0.len());
            (0..parent.0.len()).choose_multiple(&mut rng, num)
        }
    };

    trace!("Crossover points: {:?}", num_points);

    // create an infinite iterator to switch between each parent
    let mut parents = Vec::<String>::new();
    parents.push(parent.0);
    parents.push(parent.1);

    let mut p = parents.iter().cycle();
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

        last += i;
    }

    if mutation_rate > 0.0 {
        child = mutate(child, mutation_rate, alphabet, force);
    }

    debug!("Produced children: {}, {}", child.0, child.1);
    child
}

pub fn generate_genitors(population: usize, length: usize, alphabet: Vec<char>) -> Vec<String> {
    let mut v = Vec::<String>::new();
    let mut rng = rand::thread_rng();

    trace!("Creating genitors");
    for _ in 0..population {
        let mut genitor = String::new();
        for _ in 0..length {
            genitor.push(alphabet[rng.gen_range(0..alphabet.len())]);
        }

        trace!("Created genitor: {genitor}");
        v.push(genitor);
    }

    v
}

pub fn generate_population(
    fitness: impl Fn(String) -> f64,
    genitors: Option<Vec<String>>,
    population: usize, 
    length: usize,
    alphabet: Vec<char>,
    selection_method: SelectionMethod
) -> Vec<String> {
    let mut rng = rand::thread_rng();
    let mut g: Vec<(String, f64)> = match genitors {
        Some(mut g) => {
            if g.len() < population {
                g.append(&mut generate_genitors(population - g.len(), length, alphabet));
            }

            g
        },
        None => {
            generate_genitors(population, length, alphabet)
        }
    }.iter()
        .map(|genotype| (genotype.to_string(), fitness(genotype.to_string())))
        .collect();
    
    // prioritize the best performers
    g.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());

    trace!("(Genitors, fitness) = {:?}", g);

    let mut pool = Vec::<String>::new();

    match selection_method {
        SelectionMethod::Equal => {
            pool = g.iter().map(|(genotype, _)| genotype.to_string()).collect();
        },
        SelectionMethod::Remainder => {
            let avg_fit = g.iter()
                .filter(|(_, f)| f > &0.0)
                .fold(0.0, |total, (_, f)| total + f) / g.len() as f64; 
            trace!("Average fitness = {avg_fit}");

            for (genotype, fitness) in g {
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

            // TODO: What if the pool isn't full, and we're done iterating?
        },
        SelectionMethod::Replacement => {
            let avg_fit = g.iter()
                .filter(|(_, f)| f > &0.0)
                .fold(0.0, |total, (_, f)| total + f) / g.len() as f64; 
            trace!("Average fitness = {avg_fit}");

            // add the genitors randomly proportionally to their fitness
            let dist = WeightedIndex::new(g.iter().map(|(_, fitness)| {
                if fitness > &0.0 { 
                    fitness 
                } else {
                    &0.0
                }
            })).unwrap();

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
