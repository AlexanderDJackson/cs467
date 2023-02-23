use clap::Parser;
use rand::{Rng, seq::{IteratorRandom, SliceRandom}, distributions::{Distribution, WeightedIndex}};
use types::SelectionMethod;
use crate::problems::knapsack;
use simple_logger:: SimpleLogger;
use crate::types::{Args, SexMethod};
use log::{trace, debug, info, warn, error, LevelFilter};

mod types;
pub mod problems;

fn mutate(mut child: (String, String), mutation_rate: f64, alphabet: Vec<char>, force: bool) -> (String, String) {
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

fn reproduce(
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

fn generate_genitors(population: usize, length: usize, alphabet: Vec<char>) -> Vec<String> {
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

fn generate_population(
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

fn main() {
    // Create logger that defaults to info level
    // Run with env variable RUST_LOG=<desired_level> to change from default
    match SimpleLogger::new().with_level(LevelFilter::Info).env().init() {
        Ok(logger) => { logger },
        Err(e) => { eprintln!("Unable to intialize logger: {}", e); }
    }

    let args = Args::parse();

    trace!("Length of Genotype: {}", args.length);
    trace!("Generation 0 Genitors: {:?}", args.genitors);
    trace!("Reproducing Population: {}", args.population);
    trace!("Method of Reproduction: {}", args.sex_method);
    trace!("Method of Selection: {}", args.selection_method);
    trace!("Mutation Rate: {}", args.mutation_rate);
    trace!("Genetic Alphabet: {}", args.alphabet);
    trace!("Problem: {}", args.problem);
    trace!("Problem File: {}", args.file);

    let fitness;

    match args.problem {
        types::Problem::Knapsack => {
            let (max_weight, items) = match knapsack::parse_file(&args.file) {
                Some((m, i)) => { (m, i) },
                None => { panic!("Failed to parse {}", args.file); }
            };
                
            fitness = move |string: String| -> f64 { knapsack::knapsack(items.clone(), max_weight, string) };
        }
    }

    let mut pool = generate_population(
        fitness,
        if args.genitors.len() > 0 { Some(args.genitors) } else { None },
        args.population,
        args.length,
        args.alphabet.chars().collect::<Vec<char>>(),
        args.selection_method
    );
}
