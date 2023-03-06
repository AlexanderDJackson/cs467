use crate::problems::*;
use clap::{Parser, ValueEnum};
use log::{debug, error, info, trace};
use rand::{
    distributions::{Distribution, WeightedIndex},
    seq::IteratorRandom,
    thread_rng, Rng,
};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter, Result};
use std::panic;

#[derive(Clone, PartialEq, PartialOrd)]
pub struct Genotype {
    pub genotype: Vec<u8>,
    pub fitness: Fitness,
}

pub struct Generation<P: Problem> {
    pub max_generations: usize,
    pub force_mutation: bool,
    pub population: Vec<Genotype>,
    pub intermediate: Vec<Genotype>,
    pub skip: f64,
    pub mutation_rate: f64,
    pub problem: P,
    pub selection_method: SelectionMethod,
    pub sex_method: SexMethod,
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum Fitness {
    Valid(f64),
    Invalid,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum SexMethod {
    One,
    Two,
    Uniform,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum SelectionMethod {
    Equal,
    Replacement,
    Remainder,
}

impl Display for SexMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            SexMethod::One => {
                write!(f, "one")
            }
            SexMethod::Two => {
                write!(f, "two")
            }
            SexMethod::Uniform => {
                write!(f, "uniform")
            }
        }
    }
}

impl Display for SelectionMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            SelectionMethod::Equal => {
                write!(f, "equal")
            }
            SelectionMethod::Replacement => {
                write!(f, "stochastic sampling with replacement")
            }
            SelectionMethod::Remainder => {
                write!(f, "remainder stochastic sampling")
            }
        }
    }
}

impl Display for Genotype {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}: {}",
            self.genotype.iter().map(|b| *b as char).collect::<String>(),
            self.fitness
        )
    }
}

impl Display for Fitness {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Fitness::Valid(fitness) => {
                write!(f, "Valid: {fitness}")
            }
            Fitness::Invalid => {
                write!(f, "Invalid!")
            }
        }
    }
}

impl Fitness {
    fn cmp(&self, other: &Self) -> Ordering {
        let lhs = match self {
            Fitness::Valid(fitness) => fitness,
            Fitness::Invalid => &-1.0,
        };

        let rhs = match other {
            Fitness::Valid(fitness) => fitness,
            Fitness::Invalid => &-1.0,
        };

        if lhs.is_nan() {
            if rhs.is_nan() {
                Ordering::Equal
            } else {
                Ordering::Less
            }
        } else if rhs.is_nan() {
            Ordering::Greater
        } else {
            lhs.total_cmp(rhs)
        }
    }

    fn unwrap(&self) -> f64 {
        if let Fitness::Valid(fitness) = self {
            *fitness
        } else {
            panic!("Expected a Valid(f64) fitness, but found Invalid fitness");
        }
    }
}

impl Genotype {
    pub fn len(&self) -> usize {
        self.genotype.len()
    }

    pub fn new<P: Problem>(generation: &Generation<P>) -> Genotype {
        let mut rng = thread_rng();
        let mut g = Genotype {
            genotype: Vec::<u8>::with_capacity(generation.problem.len()),
            fitness: Fitness::Invalid,
        };

        while g.len() < g.genotype.capacity() {
            g.genotype.push(
                generation.problem.alphabet()
                    [rng.gen_range(0..generation.problem.alphabet().len())],
            );
        }

        g.fitness = generation.problem.fitness(&g.genotype);
        debug!("Created genitor: {g}");

        g
    }

    pub fn from<P: Problem>(&mut self, generation: &Generation<P>, g: &str) {
        self.genotype = g.as_bytes().to_vec();
        let fitness = generation.problem.fitness(&self.genotype);
        debug!("Created genitor: {self} {fitness}");
    }

    /*
    pub fn mutate<P: Problem>(&mut self, generation: Generation<P>) {
        let mut rng = rand::thread_rng();
        trace!("Force mutation: {}", generation.force_mutation);

        trace!("Testing for mutations");
        for (n, mut c) in self.genotype.chars().enumerate() {
            if rng.gen_bool(generation.mutation_rate) {
                trace!("Mutated gene {n} from: {c}");
                if generation.force_mutation {
                    let n = rng.gen_range(0..generation.problem.alphabet().len() - 1);
                    let m = generation.problem.alphabet()[n];
                    c = if m == c {
                        generation.problem.alphabet()[n + 1]
                    } else {
                        m
                    };
                } else {
                    c = generation.problem.alphabet()
                        [rng.gen_range(0..generation.problem.alphabet().len())];
                }

                trace!("to: {c}");
            }
        }
    }
    */

    pub fn reproduce<P: Problem>(&mut self, mate: &mut Genotype, generation: &Generation<P>) {
        let length = self.genotype.len();
        if length != mate.len() || length != generation.problem.len() {
            panic!("Genitor lengths are incorrect: {} != {}", self, mate);
        }

        let mut rng = rand::thread_rng();

        if rng.gen_bool(generation.skip) {
            debug!("Propogating parents: {}, {}", self, mate);
            return;
        }

        let mut num_points = match generation.sex_method {
            SexMethod::One => (1..length).choose_multiple(&mut rng, 1),
            SexMethod::Two => (1..length).choose_multiple(&mut rng, 2),
            SexMethod::Uniform => {
                let num = rng.gen_range(3..length);
                (1..length).choose_multiple(&mut rng, num)
            }
        };

        num_points.sort();

        trace!("Crossover points: {:?}", num_points);

        let mut last = 0;
        let mut toggle = true;

        for i in num_points {
            if toggle {
                self.genotype[last..i].swap_with_slice(&mut mate.genotype[last..i]);
            }

            last = i;
            toggle = !toggle;
        }

        debug!("Produced children: {self}, {mate}");
    }
}

impl<P: Problem> Generation<P> {
    pub fn from(args: Args, problem: P) -> Generation<P> {
        let mut generation = Generation {
            max_generations: args.max_generations,
            force_mutation: args.force_mutation,
            population: Vec::<Genotype>::with_capacity(args.population),
            intermediate: Vec::<Genotype>::with_capacity(args.intermediate_population),
            skip: args.skip,
            mutation_rate: args.mutation_rate,
            problem,
            selection_method: args.selection_method,
            sex_method: args.sex_method,
        };

        generation.generate_genitors();

        generation
    }

    pub fn select_genitors(&mut self) {
        let mut rng = rand::thread_rng();

        // prioritize the best performers with a reverse sort
        self.population.sort_by(|a, b| b.fitness.cmp(&a.fitness));

        // prepare the pool of genitors
        self.intermediate.clear();

        if self.population.len() > 0 {
            match self.selection_method {
                SelectionMethod::Equal => {
                    let limit = if self.intermediate.len() == 0 {
                        self.population.len() * 2
                    } else {
                        self.intermediate.len()
                    };

                    for n in 0..self.intermediate.len() {
                        self.intermediate.push(self.population[rng.gen_range(0..self.population.len())].clone());

                        if n >= limit {
                            break;
                        }
                    }
                }
                SelectionMethod::Remainder => {
                    let avg_fit = self
                        .population
                        .iter()
                        .fold(0.0, |total, f| match f.fitness {
                            Fitness::Valid(fit) => total + fit,
                            Fitness::Invalid => total,
                        })
                        / self.population.len() as f64;

                    trace!("Average fitness = {avg_fit}");

                    // loop until the pool is full
                    loop {
                        // check each genotype
                        for genotype in &self.population {
                            trace!(
                                "Length = {}, Capacity = {}",
                                self.intermediate.len(),
                                self.intermediate.capacity()
                            );
                            let mut f = genotype.fitness.unwrap() / avg_fit;
                            trace!("Fitness = {f} (avg = {avg_fit})");

                            // ensure we don't overfill the pool
                            while f > 0.0 {
                                if self.intermediate.len() == self.intermediate.capacity() {
                                    break;
                                } else if f > 1.0 {
                                    trace!("Pushing {genotype} into the pool");
                                    self.intermediate.push(genotype.clone());
                                    f -= 1.0;
                                } else {
                                    if rng.gen_bool(f) {
                                        trace!("Pushing {genotype} into the pool");
                                        self.intermediate.push(genotype.clone());
                                    }

                                    f = 0.0;
                                }
                            }
                        }

                        // we don't have a max intermediate population
                        // so just run through the genotypes once
                        if self.intermediate.len() == self.intermediate.capacity() {
                            break;
                        }
                    }
                }
                SelectionMethod::Replacement => {
                    let avg_fit = self
                        .population
                        .iter()
                        .fold(0.0, |total, f| match f.fitness {
                            Fitness::Valid(fit) => total + fit,
                            Fitness::Invalid => total,
                        })
                        / self.population.len() as f64;
                    trace!("Average fitness = {avg_fit}");

                    // add the genitors randomly proportionally to their fitness
                    // ignoring invalid genotypes
                    let dist = WeightedIndex::new(self.population.iter().map(|genotype| {
                        if let Fitness::Valid(fit) = genotype.fitness {
                            fit
                        } else {
                            0.0
                        }
                    }))
                    .expect("Unable to sample genitor population");

                    let limit = if self.intermediate.len() < 1 {
                        self.population.len() * 2
                    } else {
                        self.intermediate.len()
                    };

                    while self.intermediate.len() < limit {
                        self.intermediate
                            .push(self.population[dist.sample(&mut rng)].clone());
                    }
                }
            }
        } else {
            error!("No valid genotypes!");
            panic!("All genotypes were invalid! Either try again or supply valid ones.");
        }

        trace!("1 Intermediate population size = {}", self.intermediate.len());
        //self.population.iter().for_each(|g| trace!("{g}"));
        trace!("2 Intermediate population size = {}", self.intermediate.len());
    }

    pub fn generate_genitors(&mut self) {
        while self.population.len() < self.population.capacity() {
            self.population.push(Genotype::new(&self));
        }
    }

    pub fn generate_generation(&mut self, num_generation: usize) {
        let mut rng = rand::thread_rng();

        if !self.population.iter().fold(true, |good, genotype| {
            good && genotype.len() == self.problem.len()
        }) {
            panic!("Genitor genotype is incorrect length!");
        }

        trace!("Population size = {}", self.population.len());

        // fill the intermediate population
        self.select_genitors();

        // clear the genitors
        self.population.clear();

        while self.population.len() < self.population.capacity() {
            trace!("Population size = {}", self.population.len());
            trace!("Itermediate population size = {}", self.intermediate.len());

            let mut genotype = (
                self.intermediate[rng.gen_range(0..self.intermediate.len())].clone(),
                self.intermediate[rng.gen_range(0..self.intermediate.len())].clone(),
            );

            genotype.0.reproduce(&mut genotype.1, self);

            self.population.push(genotype.0);

            if self.population.len() < self.population.capacity() {
                self.population.push(genotype.1);
            }
        }

        // prioritize the best performers with a reverse sort
        self.population.sort_by(|a, b| b.fitness.cmp(&a.fitness));

        info!("Generation {num_generation}");
        info!("--------------------------");

        for g in &self.population {
            let fitness = self.problem.fitness(&g.genotype);
            info!("{g}: {fitness}");
        }

        info!("--------------------------");
        info!("");
    }
}

/// Genetic algorithm to generate optimal solutions
#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
pub struct Args {
    /// The maximum number of generations
    #[arg(short = 'e', long, default_value_t = 100)]
    pub max_generations: usize,

    /// Force mutation if one occurs
    #[arg(short, long, default_value_t = false)]
    pub force_mutation: bool,

    /// The file needed for whatever problem
    #[arg(long)]
    pub file: Option<String>,

    /// The initial population of genitors
    #[arg(short, long, num_args = 1..)]
    pub genitors: Vec<String>,

    /// The number of genotypes in each intermediate population
    #[arg(short, long, default_value_t = 100)]
    pub intermediate_population: usize,

    /// Chance of skipping reproduction and adding genitors to next generation
    #[arg(short = 'k', long, default_value_t = 0.1)]
    pub skip: f64,

    /// Length of genotypes
    #[arg(short, long, required = true)]
    pub length: usize,

    /// The mutation rate
    #[arg(short, long, default_value_t = 0.01)]
    pub mutation_rate: f64,

    /// The number of genitors in each population
    #[arg(short, long, default_value_t = 50)]
    pub population: usize,

    /// The problem for which to generate solutions
    #[arg(short = 'r', long, value_enum, default_value_t = ProblemType::Knapsack)]
    pub problem: ProblemType,

    /// The method of selection used to produce genitors from a population
    #[arg(short, long, value_enum, default_value_t = SelectionMethod::Equal)]
    pub selection_method: SelectionMethod,

    /// The method used to produce subsequent generations from genitors
    #[arg(short = 'x', long, value_enum, default_value_t = SexMethod::Uniform)]
    pub sex_method: SexMethod,
}
