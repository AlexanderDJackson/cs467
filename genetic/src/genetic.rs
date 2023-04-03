use crate::problems::*;
use clap::{Parser, ValueEnum};
use log::{debug, error, info, trace};
use rand::{
    distributions::{Distribution, WeightedIndex},
    seq::IteratorRandom,
    Rng,
};
use std::{
    fmt::{Display, Formatter, Result},
    panic,
};

#[derive(Clone, Debug)]
pub struct Genotype {
    pub fitness: Fitness,
    pub genotype: Vec<u8>,
}

pub struct Generation {
    pub force_create: bool,
    pub detect_crowding: f64,
    pub max_generations: usize,
    pub force_mutation: bool,
    pub population: Vec<Genotype>,
    pub intermediate: Vec<Genotype>,
    pub skip: f64,
    pub mutation_rate: f64,
    pub problem: Box<dyn Problem>,
    pub selection_method: SelectionMethod,
    pub sex_method: SexMethod,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
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
                write!(f, "{fitness}")
            }
            Fitness::Invalid => {
                write!(f, "Invalid!")
            }
        }
    }
}

/*
impl Ord for Fitness {
    fn cmp(&self, other: &Self) -> Ordering {
        let lhs = match self {
            Fitness::Valid(fitness) => fitness,
            Fitness::Invalid => &-1.0,
        };

        let rhs = match other {
            Fitness::Valid(fitness) => fitness,
            Fitness::Invalid => &-1.0,
        };

        lhs.total_cmp(rhs)
    }
}

impl PartialOrd for Fitness {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let lhs = match self {
            Fitness::Valid(fitness) => fitness,
            Fitness::Invalid => &-1.0,
        };

        let rhs = match other {
            Fitness::Valid(fitness) => fitness,
            Fitness::Invalid => &-1.0,
        };

        Some(lhs.total_cmp(rhs))
    }
}

impl PartialEq for Fitness {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Fitness::Valid(fitness) => match other {
                Fitness::Valid(other_fitness) => fitness == other_fitness,
                Fitness::Invalid => false,
            },
            Fitness::Invalid => match other {
                Fitness::Valid(_) => false,
                Fitness::Invalid => true,
            },
        }
    }
}
*/

impl Fitness {
    pub fn unwrap(&self) -> f64 {
        match self {
            Fitness::Valid(fitness) => *fitness,
            Fitness::Invalid => panic!("Fitness is invalid!"),
        }
    }
}

/*
impl Ord for Genotype {
    fn cmp(&self, other: &Self) -> Ordering {
        self.fitness.cmp(&other.fitness)
    }
}

impl PartialOrd for Genotype {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.fitness.partial_cmp(&other.fitness)
    }
}

impl PartialEq for Genotype {
    fn eq(&self, other: &Self) -> bool {
        self.fitness == other.fitness
    }
}
*/

impl Genotype {
    pub fn len(&self) -> usize {
        self.genotype.len()
    }

    pub fn new(generation: &Generation) -> Genotype {
        generation
            .problem
            .generate_genotype(generation.force_create)
    }

    pub fn from(genotype: Vec<u8>, fitness: Fitness) -> Genotype {
        Genotype { genotype, fitness }
    }

    pub fn reproduce(&mut self, mate: &mut Genotype, generation: &Generation) {
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

        generation
            .problem
            .mutate(generation.mutation_rate, generation.force_mutation, self);
        generation
            .problem
            .mutate(generation.mutation_rate, generation.force_mutation, mate);

        debug!("Produced children: {self}, {mate}");
    }
}

impl Generation {
    pub fn from(args: Args) -> Generation {
        let mut generation = Generation {
            force_create: args.force_create,
            detect_crowding: args.detect_crowding,
            max_generations: args.max_generations,
            force_mutation: args.force_mutation,
            population: Vec::<Genotype>::with_capacity(args.population),
            intermediate: Vec::<Genotype>::with_capacity(args.intermediate_population),
            skip: args.skip,
            mutation_rate: args.mutation_rate,
            problem: match args.problem {
                ProblemType::Knapsack => {
                    Box::new(knapsack::Knapsack::new(args.file).expect("Failed to create problem"))
                }
                ProblemType::Stocks => {
                    Box::new(stocks::Market::new(args.file).expect("Failed to create problem"))
                }
            },
            selection_method: args.selection_method,
            sex_method: args.sex_method,
        };

        for g in args.genitors {
            let fit = generation.problem.fitness(&g);
            trace!("Pushing {}", Genotype::from(g.clone(), fit.clone()));
            generation.population.push(Genotype::from(g, fit));
        }

        if !args.evaluate && generation.population.len() < generation.population.capacity() {
            generation.generate_genitors();
        }

        generation.population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).expect("Illegal fitness"));

        generation
    }

    pub fn best(&self) -> Option<&Genotype> {
        if self.population.len() > 0 {
            Some(
                self.population
                    .iter()
                    .max_by(|a, b| a.fitness.partial_cmp(&b.fitness).expect("Illegal fitness"))
                    .unwrap(),
            )
        } else {
            None
        }
    }

    pub fn select_genitors(&mut self) {
        let mut rng = rand::thread_rng();

        // prioritize the best performers with a reverse sort
        self.population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).expect("Illegal fitness"));

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

                    for n in 0..self.intermediate.capacity() {
                        self.intermediate
                            .push(self.population[rng.gen_range(0..self.population.len())].clone());

                        if n >= limit {
                            break;
                        }
                    }
                }
                SelectionMethod::Remainder => {
                    let sum = self
                        .population
                        .iter()
                        .fold(0.0, |total, f| match f.fitness {
                            Fitness::Valid(fit) => total + fit,
                            Fitness::Invalid => total,
                        });
                    let avg_fit = if sum > 0.0 {
                        sum / self.population.len() as f64
                    } else {
                        0.0000000000001
                    };

                    trace!("Average fitness = {avg_fit}");

                    // loop until the pool is full
                    loop {
                        let mut pushed = 0;
                        // check each genotype
                        for genotype in &self.population {
                            match genotype.fitness {
                                Fitness::Valid(fit) => {
                                    let mut f = fit / avg_fit;
                                    debug!("Fitness = {f} (avg = {avg_fit})");

                                    // ensure we don't overfill the pool
                                    while f > 0.0 {
                                        if self.intermediate.len() == self.intermediate.capacity() {
                                            break;
                                        } else if f > 1.0 {
                                            debug!("Pushing {genotype} into the pool");
                                            self.intermediate.push(genotype.clone());
                                            pushed += 1;
                                            f -= 1.0;
                                        } else {
                                            if rng.gen_bool(f) {
                                                debug!("Pushing {genotype} into the pool");
                                                self.intermediate.push(genotype.clone());
                                                pushed += 1;
                                            }

                                            f = 0.0;
                                        }
                                    }
                                }
                                Fitness::Invalid => continue,
                            }
                        }

                        if pushed == 0 {
                            self.intermediate
                                .push(self.population[rng.gen_range(0..self.population.len())].clone());
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
                    let dist = match WeightedIndex::new(self.population.iter().map(|genotype| {
                        if let Fitness::Valid(fit) = genotype.fitness {
                            fit.clamp(0.01, f64::MAX)
                        } else {
                            0.01
                        }
                    })) {
                        Ok(d) => d,
                        Err(_) => {
                            panic!("No valid genotypes to select from!");
                        }
                    };

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

        // prioritize the best performers with a reverse sort
        self.intermediate.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).expect("Illegal fitness"));

        debug!("Intermediate Population:");
        self.intermediate.iter().for_each(|g| debug!("\t{g}"));
    }

    pub fn generate_genitors(&mut self) {
        debug!("Generating genitors");
        while self.population.len() < self.population.capacity() {
            self.population.push(Genotype::new(&self));
        }
    }

    fn detected_crowding(&mut self) -> bool {
        // iterate over consecutive pairs of genotypes
        self.intermediate[0..self.intermediate.len() - 1]
            .iter()
            .step_by(2)
            .zip(self.intermediate[1..].iter().step_by(2))
            // count the number of matching genes
            .fold(0, |crowded, (a, b)| {
                crowded
                    + a.genotype
                        .iter()
                        .zip(b.genotype.iter())
                        .fold(0, |same, (a, b)| if a == b { same + 1 } else { same })
            })
            // average the number of matching genes
            / (self.intermediate.len() / 2)
            // return true if the average is greater than 80%
            > (self.intermediate[0].len() as f64 * 0.8) as usize
    }

    pub fn generate_generation(&mut self, num_generation: usize) {
        let mut rng = rand::thread_rng();

        if !self.population.iter().fold(true, |good, genotype| {
            good && genotype.len() == self.problem.len()
        }) {
            panic!("Genitor genotype is incorrect length!");
        }

        // fill the intermediate population
        self.select_genitors();

        let old = self.mutation_rate;

        if self.detect_crowding > 0.0 && self.detected_crowding() {
            debug!("Crowding detected! Ramping up mutation rate for a generation.");
            self.mutation_rate = 0.2;
        }

        // clear the genitors
        self.population.clear();

        while self.population.len() < self.population.capacity() {
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
        self.population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).expect("Illegal fitness"));

        /*
        for i in self.population[..].iter() {
            println!("{i}");
        }
        */

        self.mutation_rate = old;

        info!("--------------------------");
        info!("Generation {num_generation}");

        let mut invalid = 0;

        for g in &self.population {
            if let Fitness::Valid(_) = g.fitness {
                info!("{}", self.problem.format(g));
            } else {
                invalid += 1;
                debug!("{g}");
            }
        }

        info!("{invalid} invalid genotypes");
        info!("--------------------------");
    }
}

/// Genetic algorithm to generate optimal solutions
#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
pub struct Args {
    /// Print n best genotypes from all generations
    #[arg(short, long, default_value_t = 1)]
    pub best: usize,

    /// Force create genitors until valid
    #[arg(short = 'c', long, default_value_t = false)]
    pub force_create: bool,

    /// Detect crowding and ramp up mutation rate
    #[arg(short, long, default_value_t = 0.0)]
    pub detect_crowding: f64,

    /// Evaluate the fitness of the given genitors
    #[arg(short, long, default_value_t = false)]
    pub evaluate: bool,

    /// Force mutation if one occurs
    #[arg(short, long, default_value_t = false)]
    pub force_mutation: bool,

    /// The file needed for whatever problem
    #[arg(long, required = true, num_args = 1..)]
    pub file: Vec<String>,

    /// The initial population of genitors
    #[arg(short, long, num_args = 1..)]
    pub genitors: Vec<Vec<u8>>,

    /// The number of genotypes in each intermediate population
    #[arg(short, long, default_value_t = 100)]
    pub intermediate_population: usize,

    /// Chance of skipping reproduction and adding genitors to next generation
    #[arg(short = 'k', long, default_value_t = 0.1)]
    pub skip: f64,

    /// The mutation rate
    #[arg(short, long, default_value_t = 0.01)]
    pub mutation_rate: f64,

    /// The maximum number of generations
    #[arg(short = 'M', long, default_value_t = 100)]
    pub max_generations: usize,

    /// Progress bar
    #[arg(short = 'o', long, default_value_t = false)]
    pub progress: bool,

    /// The number of genitors in each population
    #[arg(short, long, default_value_t = 50)]
    pub population: usize,

    /// The problem for which to generate solutions
    #[arg(short = 'r', long, required = true, value_enum, default_value_t = ProblemType::Knapsack)]
    pub problem: ProblemType,

    /// The method of selection used to produce genitors from a population
    #[arg(short, long, value_enum, default_value_t = SelectionMethod::Equal)]
    pub selection_method: SelectionMethod,

    /// The method used to produce subsequent generations from genitors
    #[arg(short = 'x', long, value_enum, default_value_t = SexMethod::Uniform)]
    pub sex_method: SexMethod,
}

#[cfg(test)]
pub mod tests {
    #[test]
    fn test_fitness_cmp() {
        use super::Fitness;

        assert!(Fitness::Valid(1.0) > Fitness::Valid(0.0));
        assert!(Fitness::Valid(0.0) < Fitness::Valid(1.0));
        assert!(Fitness::Valid(0.0) == Fitness::Valid(0.0));
        assert!(Fitness::Valid(1.0) != Fitness::Valid(0.0));
        assert!(Fitness::Valid(0.0) != Fitness::Valid(1.0));
        assert!(Fitness::Valid(0.0) != Fitness::Invalid);
        assert!(Fitness::Invalid != Fitness::Valid(0.0));
        assert!(Fitness::Invalid == Fitness::Invalid);
    }
}
