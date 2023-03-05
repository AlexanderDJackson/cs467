use crate::genetic::*;
use clap::ValueEnum;
use rand::Rng;
use std::fmt::{Display, Formatter, Result};

pub trait Problem {
    fn fitness(&self, genotype: &str) -> Fitness;
    fn mutate(&self, args: &Args, genotype: &mut str);
    fn format(&self, g: &Genotype, v: bool) -> &str;
    fn get_alphabet(&self) -> Vec<char>;
    fn new(file_name: &String) -> Option<Self>
    where
        Self: Sized;
}

#[derive(Clone, Debug, ValueEnum)]
pub enum ProblemType {
    Knapsack,
}

impl Display for ProblemType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ProblemType::Knapsack => {
                write!(f, "knapsack")
            }
        }
    }
}

pub mod knapsack {

    use crate::problems::*;
    use log::{debug, error, trace};
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    pub struct Knapsack {
        pub alphabet: Vec<char>,
        pub items: &'static Vec<(usize, usize)>,
        pub max_weight: usize,
    }

    impl Problem for Knapsack {
        fn fitness(&self, g: &str) -> Fitness {
            let (weight, value) = self
                .items
                .iter()
                .zip(g.chars())
                .filter(|(_, b)| b == &'1')
                .fold((0, 0), |(weight, value), ((w, v), _)| {
                    (weight + w, value + v)
                });

            if weight > self.max_weight {
                Fitness::Invalid
            } else {
                let fitness = value as f64 / self.max_weight as f64;

                assert!(fitness.is_finite());
                assert!(fitness.is_sign_positive());

                // simple sigmoid function
                Fitness::Valid(fitness / (1.0 + fitness))
            }
        }

        fn format(&self, g: &Genotype, v: bool) -> &str {
            let (weight, value) = self
                .items
                .iter()
                .zip(g.genotype.chars())
                .filter(|(_, b)| b == &'1')
                .fold((0, 0), |(weight, value), ((w, v), _)| {
                    (weight + w, value + v)
                });

            if v {
                &format!(
                    "{:?}: (weight: {}, value: {}, fitness: {})",
                    self.items,
                    weight,
                    value,
                    self.fitness(g.genotype)
                )
                .to_string()
            } else {
                &format!(
                    "weight: {}, value: {}, fitness: {}",
                    weight,
                    value,
                    self.fitness(g.genotype)
                )
                .to_string()
            }
        }

        fn get_alphabet(&self) -> Vec<char> {
            self.alphabet
        }

        fn mutate(&self, args: &Args, g: &mut str) {
            let mut rng = rand::thread_rng();
            trace!("Force mutation: {}", args.force_mutation);

            trace!("Testing for mutations");
            for (n, mut c) in g.chars().enumerate() {
                if rng.gen_bool(args.mutation_rate) {
                    trace!("Mutated gene {n} from: {c}");
                    if args.force_mutation {
                        let n = rng.gen_range(0..self.get_alphabet().len() - 1);
                        let m = self.get_alphabet()[n];
                        c = if m == c {
                            self.get_alphabet()[n + 1]
                        } else {
                            m
                        };
                    } else {
                        c = self.get_alphabet()[rng.gen_range(0..self.get_alphabet().len())];
                    }

                    trace!("to: {c}");
                }
            }
        }

        fn new(file_name: &String) -> Option<Knapsack> {
            let (max_weight, v) = parse_file(file_name)?;

            Some(Knapsack {
                alphabet: vec!['0', '1'],
                items: &v,
                max_weight,
            })
        }
    }

    fn parse_file(file_name: &String) -> Option<(usize, Vec<(usize, usize)>)> {
        trace!("Reading {file_name}");
        let file = match File::open(&file_name) {
            Ok(f) => {
                debug!("Successfully read {}", file_name);
                f
            }
            Err(e) => {
                panic!("Error reading {}: {}", file_name, e);
            }
        };

        let reader = BufReader::new(file);
        trace!("Created BufReader for {file_name}");
        let mut v = Vec::<(usize, usize)>::new();
        trace!("Created vector for weight value pairs");
        let mut max_weight: usize = 0;
        trace!("max_weight = 0");

        for (n, line) in reader.lines().enumerate() {
            if let Ok(l) = line {
                trace!("Read line {n}: {l}");

                let temp: Vec<String> = l
                    .replace(" ", "") // remove spaces to yield CSV strings
                    .split(',') // split the string on the commas
                    .map(|l: &str| l.to_string()) // convert the &str's to owned String's
                    .collect(); // collect them as a vector

                trace!("Processed line {n}: {:?}", temp);

                if temp.len() > 2 {
                    if let Ok(x) = temp[1].parse::<usize>() {
                        if let Ok(y) = temp[2].parse::<usize>() {
                            trace!("Parsed {}: String as {x}: usize", temp[1]);
                            trace!("Parsed {}: String as {y}: usize", temp[2]);
                            v.push((x, y));
                            trace!("Pushed {:?} into v", (x, y));
                        } else {
                            error!("Failed to parse {}", temp[2]);
                        }
                    } else {
                        error!("Failed to parse {}", temp[1]);
                    }
                } else if temp.len() == 2 {
                    max_weight = match temp[1].parse::<usize>() {
                        Ok(num) => {
                            trace!("Parsed {}: String as {}: usize", temp[1], num);
                            trace!("max_weight = {num}");
                            num
                        }
                        Err(e) => {
                            panic!("Failed to get weight from {file_name}: {e}");
                        }
                    }
                }
            } else {
                error!("Failed to read line {}", n);
                return None;
            }
        }

        if max_weight == 0 {
            panic!("Failed to get weight from {file_name}");
        }

        trace!("Returning Some({:?})", (max_weight, &v));
        Some((max_weight, v))
    }
}
