use crate::{
    genetic::*,
    problems::*,
};
use clap::Parser;
use log::{trace, info, LevelFilter};
use simple_logger::SimpleLogger;

pub mod genetic;
pub mod problems;

//#[cfg(test)]
//mod tests;

fn main() {
    /*
    panic::set_hook(Box::new(|panic_info| {
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            println!("Unable to continue: {s}");
        } else {
            println!("Unable to continue");
        }
    }));
    */

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

    let pb = if !args.progress {
        indicatif::ProgressBar::hidden()
    } else {
        indicatif::ProgressBar::new(args.max_generations as u64)
    };


    trace!("Arguments: {:?}", args);

    let mut generation = Generation::from(args);
    let mut best = generation.population[0].clone();

    info!("Generation: 0 Best: {best}");

    for i in 0..generation.population.len() {
        trace!("\t{}", generation.population[i]);
    }

    info!("Generation: 0 Best: {best}");

    for i in 1..generation.max_generations {
        generation.generate_generation(i);

        pb.inc(1);

        let new = &generation.population[0];

        if new.fitness > best.fitness {
            best = new.clone();
        }

        info!("Generation: {i} Best: {best}");

        for i in 0..generation.population.len() {
            trace!("\t{}", generation.population[i]);
        }
    }

    pb.finish();

    println!("Best Solution: {}", generation.problem.format(&best));
}
