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
pub struct Genotype<'a> {
    pub genotype: &'a str,
    pub fitness: Fitness,
}

pub struct Generation<'a, P: Problem> {
    pub max_generations: usize,
    pub force_mutation: bool,
    pub genitors: Vec<Genotype<'a>>,
    pub intermediate_population: usize,
    pub skip: f64,
    pub length: usize,
    pub mutation_rate: f64,
    pub population: usize,
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

impl Display for Genotype<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.genotype)
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

        let rhs = match self {
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

impl Genotype<'_> {
    pub fn len(&self) -> usize {
        self.genotype.len()
    }

    pub fn new<'a, P: Problem>(generation: &Generation<P>) -> Genotype<'a> {
        let mut g = String::new();
        let mut rng = thread_rng();

        for _ in 0..generation.length {
            g.push(
                generation.problem.get_alphabet()
                    [rng.gen_range(0..generation.problem.get_alphabet().len())],
            );
        }

        let fitness = generation.problem.fitness(&g);
        debug!("Created genitor: {g} {fitness}");

        Genotype {
            genotype: &g.to_string(),
            fitness,
        }
    }

    pub fn from<'a, P: Problem>(generation: &Generation<P>, g: String) -> Genotype<'a> {
        let fitness = generation.problem.fitness(&g);
        debug!("Created genitor: {g} {fitness}");

        Genotype {
            genotype: &g.to_string(),
            fitness,
        }
    }

    pub fn mutate<P: Problem>(&mut self, generation: Generation<P>) {
        let mut rng = rand::thread_rng();
        trace!("Force mutation: {}", generation.force_mutation);

        trace!("Testing for mutations");
        for (n, mut c) in self.genotype.chars().enumerate() {
            if rng.gen_bool(generation.mutation_rate) {
                trace!("Mutated gene {n} from: {c}");
                if generation.force_mutation {
                    let n = rng.gen_range(0..generation.problem.get_alphabet().len() - 1);
                    let m = generation.problem.get_alphabet()[n];
                    c = if m == c {
                        generation.problem.get_alphabet()[n + 1]
                    } else {
                        m
                    };
                } else {
                    c = generation.problem.get_alphabet()
                        [rng.gen_range(0..generation.problem.get_alphabet().len())];
                }

                trace!("to: {c}");
            }
        }
    }

    pub fn reproduce<'a, 'b, P: Problem>(
        self,
        mate: Genotype<'a>,
        generation: &Generation<P>,
    ) -> (Genotype<'b>, Genotype<'a>) {
        let length = self.len();
        if length != mate.len() || length != generation.length {
            panic!("Genitor lengths are incorrect: {} != {}", self, mate);
        }

        let mut rng = rand::thread_rng();

        if rng.gen_bool(generation.skip) {
            debug!("Returning parents: {}, {}", self, mate);
            return (self, mate);
        }

        // create an infinite iterator to switch between each parent
        let parents = vec![self.genotype, mate.genotype];
        let mut p = parents.iter().cycle();

        let mut child = (String::new(), String::new());
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

        for i in num_points {
            child.0 += p.next().unwrap().get(last..i).unwrap();
            child.1 += p.next().unwrap().get(last..i).unwrap();

            trace!("Child 0: {}", child.0);
            trace!("Child 1: {}", child.1);

            last = i;
            p.next();
        }

        let temp = &p.next().unwrap();
        trace!(
            "Taking {} alleles from parent {} for child 0: {}",
            length - last,
            temp,
            temp[last..length].to_string()
        );
        child.0 += &temp[last..length];

        let temp = &p.next().unwrap();
        trace!(
            "Taking {} alleles from parent {} for child 0: {}",
            length - last,
            temp,
            temp[last..length].to_string()
        );
        child.1 += &temp[last..length];

        debug!("Produced children: {}, {}", child.0, child.1);
        (
            Genotype {
                genotype: &child.0.to_string(),
                fitness: generation.problem.fitness(&child.0.to_string()),
            },
            Genotype {
                genotype: &child.1.to_string(),
                fitness: generation.problem.fitness(&child.1.to_string()),
            },
        )
    }
}

impl<'a, P: Problem> Generation<'a, P> {
    pub fn from(args: Args, problem: P) -> Generation<'a, P> {
        let generation = Generation {
            max_generations: args.max_generations,
            force_mutation: args.force_mutation,
            genitors: Vec::<Genotype<'a>>::new(),
            intermediate_population: args.intermediate_population,
            skip: args.skip,
            length: args.length,
            mutation_rate: args.mutation_rate,
            population: args.population,
            problem,
            selection_method: args.selection_method,
            sex_method: args.sex_method,
        };

        generation.generate_genitors();

        generation
    }

    pub fn select_genitors(&self) -> Vec<Genotype<'a>> {
        let mut rng = rand::thread_rng();
        let mut pool = Vec::<Genotype<'a>>::new();
        let mut g: Vec<Genotype> = self
            .genitors
            .iter()
            .map(|genotype| Genotype::from(self, genotype.to_string()))
            .filter(|g| match g.fitness {
                Fitness::Valid(_) => true,
                Fitness::Invalid => false,
            })
            .collect();

        // prioritize the best performers with a reverse sort
        g.sort_by(|a, b| b.fitness.cmp(&a.fitness));

        trace!("Genitors:");
        g.iter().for_each(|g| trace!("{g}"));

        if g.len() > 0 {
            match self.selection_method {
                SelectionMethod::Equal => {
                    let mut limit = self.intermediate_population;
                    if limit == 0 {
                        limit *= 2;
                    }

                    while pool.len() < limit {
                        pool.push(g[rng.gen_range(0..g.len())]);
                    }
                }
                SelectionMethod::Remainder => {
                    let avg_fit = g.iter().fold(0.0, |total, f| match f.fitness {
                        Fitness::Valid(fit) => total + fit,
                        Fitness::Invalid => total,
                    }) / g.len() as f64;
                    trace!("Average fitness = {avg_fit}");

                    // loop until the pool is full
                    loop {
                        // check each genotype
                        for genotype in &g {
                            let mut f = genotype.fitness.unwrap() / avg_fit;

                            // ensure we don't overfill the pool
                            while (self.intermediate_population == 0
                                || pool.len() < self.intermediate_population)
                                && f > 0.0
                            {
                                if f > 1.0 {
                                    trace!("Pushing {genotype} into the pool");
                                    pool.push(genotype.clone());
                                    f = -1.0;
                                } else {
                                    if rng.gen_bool(f) {
                                        trace!("Pushing {genotype} into the pool");
                                        pool.push(genotype.clone());
                                    }

                                    f = 0.0;
                                }
                            }
                        }

                        // we don't have a max intermediate population
                        // so just run through the genotypes once
                        if self.intermediate_population == 0 {
                            break;
                        }
                    }
                }
                SelectionMethod::Replacement => {
                    let avg_fit = g.iter().fold(0.0, |total, f| match f.fitness {
                        Fitness::Valid(fit) => total + fit,
                        Fitness::Invalid => total,
                    }) / g.len() as f64;
                    trace!("Average fitness = {avg_fit}");

                    // add the genitors randomly proportionally to their fitness
                    // ignoring invalid genotypes
                    let dist = WeightedIndex::new(g.iter().map(|genotype| {
                        if let Fitness::Valid(fit) = genotype.fitness {
                            fit
                        } else {
                            0.0
                        }
                    }))
                    .expect("Unable to sample genitor population");

                    let limit = if self.intermediate_population < 1 {
                        self.population * 2
                    } else {
                        self.intermediate_population
                    };

                    while pool.len() < limit {
                        let temp = g[dist.sample(&mut rng)];
                        trace!("Pushing {temp} into the pool");
                        pool.push(temp);
                    }
                }
            }
        } else {
            error!("No valid genotypes!");
            panic!("All genotypes were invalid! Either try again or supply valid ones.");
        }

        pool.iter().for_each(|g| trace!("{g}"));
        pool
    }

    pub fn generate_genitors(&mut self) {
        let mut rng = rand::thread_rng();

        trace!("Creating genitors");
        while self.genitors.len() < self.population {
            self.genitors.push(Genotype::new(self));
        }
    }

    pub fn generate_generation(&mut self, num_generation: usize) {
        let mut rng = rand::thread_rng();

        if self.genitors.len() == 0 || self.genitors.len() < self.population {
            self.generate_genitors();
        }

        if !self
            .genitors
            .iter()
            .fold(true, |good, genotype| good && genotype.len() == self.length)
        {
            panic!("Genitor genotype is incorrect length!");
        }

        let pool = self.select_genitors();

        self.genitors.clear();

        while self.genitors.len() < self.population {
            let child = pool[rng.gen_range(0..pool.len())]
                .reproduce(pool[rng.gen_range(0..pool.len())], self);

            trace!("{} made it into self.genitors generation", child.0);
            self.genitors.push(child.0);

            if self.genitors.len() > self.population {
                trace!("{} made it into next generation", child.1);
                self.genitors.push(child.1);
            }
        }

        // prioritize the best performers with a reverse sort
        self.genitors.sort_by(|a, b| b.fitness.cmp(&a.fitness));

        info!("Generation {num_generation}");
        info!("--------------------------");

        for g in &self.genitors {
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
