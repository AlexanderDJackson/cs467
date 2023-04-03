use crate::{genetic::*, problems::*};
use clap::Parser;
use ctrlc;
use indicatif::ProgressStyle;
use log::{debug, info, trace, LevelFilter};
use simple_logger::SimpleLogger;
use std::{collections::VecDeque, sync::mpsc::channel};

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
    let num = args.best;
    let histories = args.file.len();

    let mut generation = Generation::from(args);

    if evaluate {
        println!("Genotype\tAverage \tTotal");
        for g in generation.population {
            println!(
                "{}\t${:.2}\t${:.2}",
                g.genotype.iter().map(|x| *x as char).collect::<String>(),
                g.fitness.unwrap() / histories as f64,
                g.fitness.unwrap()
            );
        }
    } else {
        assert!(num > 0, "Number of best solutions must be greater than 0");

        let mut best = VecDeque::<Genotype>::with_capacity(num);
        for i in 0..num {
            best.push_back(generation.population[i].clone());
        }

        let (tx, rx) = channel();
        ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
            .expect("Error setting Ctrl-C handler");

        info!("Generation: 0 Best: {}", best.back().unwrap());

        pb.set_message(format!("${:.2}", best[0].fitness.unwrap()));

        for i in generation.population.iter() {
            debug!("\t{i}");
        }

        for i in 1..generation.max_generations {
            generation.generate_generation(i);

            pb.inc(1);

            let new = &generation.population[0];

            //pb.println(format!("{}", generation.problem.format(&new)));

            if new.fitness > best.back().unwrap().fitness {
                if best.len() == num {
                    best.pop_front();
                }

                best.push_back(new.clone());
                pb.set_message(format!("${:.2}", new.fitness.unwrap()));
            }

            info!("Generation: {i} Best: {}", best.back().unwrap());

            for i in generation.population.iter() {
                debug!("\t{i}");
            }

            if rx.try_recv().is_ok() {
                println!();

                for i in best.iter().rev() {
                    println!("{}", generation.problem.format(&i));
                }
            }
        }

        pb.finish();

        for _ in 0..best.len() {
            println!("{}", generation.problem.format(&best.pop_back().unwrap()));
        }
    }
}
