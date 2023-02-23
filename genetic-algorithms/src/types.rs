use std::fmt;
use clap::{self, Parser, ValueEnum};

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
