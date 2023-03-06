use crate::genetic::*;
use crate::knapsack::*;
use crate::problems::*;
use clap::Parser;
use log::{trace, LevelFilter};
use simple_logger::SimpleLogger;

pub mod genetic;
pub mod problems;

//#[cfg(test)]
//mod tests;

fn main() {
    // Create logger that defaults to info level
    // Run with env variable RUST_LOG=<desired_level> to change from default
    match SimpleLogger::new()
        .with_level(LevelFilter::Warn)
        .env()
        .init()
    {
        Ok(logger) => logger,
        Err(e) => {
            eprintln!("Unable to intialize logger: {}", e);
        }
    }

    let args = Args::parse();
    let file_name = args.file.clone().expect("Unable to read filename");

    trace!("Arguments: {:?}", args);

    let mut generation = match &args.problem {
        ProblemType::Knapsack => Generation::<Knapsack>::from(
            args,
            knapsack::Knapsack::new(&file_name).expect("Failed to create problem"),
        ),
    };

    generation.generate_generation(0);

    let mut best = generation.population[0].clone();

    for i in 1..generation.max_generations {
        generation.generate_generation(i);

        let new = &generation.population[0];

        if new.fitness > best.fitness {
            best = new.clone();
        }
    }

    println!("Best Solution: {best}");
}
