use crate::{
    genetic::*,
    problems::*,
};
use clap::Parser;
use indicatif::ProgressStyle;
use log::{trace, info, LevelFilter, debug};
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

    pb.set_style(ProgressStyle::with_template("{msg} {wide_bar} {pos}/{len}").unwrap());

    trace!("Arguments: {:?}", args);

    let evaluate = args.evaluate;

    let mut generation = Generation::from(args);

    if evaluate {
        for i in 0..generation.population.len() {
            println!("{}", generation.population[i]);
        }
    } else {
        let mut best = generation.best().unwrap().clone();

        info!("Generation: 0 Best: {best}");

        pb.set_message(format!("{:.2}", best.fitness.unwrap()));

        for i in 0..generation.population.len() {
            debug!("\t{}", generation.population[i]);
        }

        for i in 1..generation.max_generations {
            generation.generate_generation(i);

            pb.inc(1);

            let new = generation.best().unwrap();

            if new.fitness > best.fitness {
                best = new.clone();
                pb.set_message(format!("{:.2}", best.fitness.unwrap()));
            }

            info!("Generation: {i} Best: {best}");

            for i in 0..generation.population.len() {
                debug!("\t{}", generation.population[i]);
            }
        }

        pb.finish();

        println!("Best Solution: {}", generation.problem.format(&best));
    }
}
