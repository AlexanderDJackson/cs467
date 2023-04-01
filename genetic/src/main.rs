use crate::{genetic::*, problems::*};
use rayon::prelude::*;
use clap::Parser;
use ctrlc;
use indicatif::ProgressStyle;
use log::{debug, info, trace, LevelFilter};
use simple_logger::SimpleLogger;
use std::{sync::mpsc::channel, thread, collections::VecDeque};

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
        indicatif::ProgressBar::new(300 * 300 * 300) //args.max_generations as u64)
    };

    pb.set_style(ProgressStyle::with_template("{msg} {wide_bar} {pos}/{len}").unwrap());

    trace!("Arguments: {:?}", args);

    let evaluate = args.evaluate;
    let num = args.best;

    let mut generation = Generation::from(args);

    if evaluate {
        /*
        for i in 0..generation.population.len() {
            println!("{}", generation.problem.format(&generation.population[i]));
        }
        */
    } else {
        assert!(num > 0, "Number of best solutions must be greater than 0");

        let mut best = VecDeque::<Genotype>::with_capacity(num);
        /*

        let guh = "m002|e010|s101".bytes().collect();
        let fit = generation.problem.fitness(&guh);
        while best.len() < num {
            best.push_back(Genotype::from(guh.clone(), fit.clone()));
        }

        pb.set_message(format!("${:.2}", best.front().unwrap().fitness.unwrap()));
        let operators = [ b"||" ];
        let strategies = [ b"ems" ];
        let mut days = [0, 0, 0];

        let (tx, rx) = channel();

        ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
            .expect("Error setting Ctrl-C handler");

        // brute force search all strategies
        for op in operators {
            for strat in strategies.iter() {
                loop {
                    let g = format!(
                        "{}{:03}{}{}{:03}{}{}{:03}",
                        strat[0] as char,
                        days[0],
                        op[0] as char,
                        strat[1] as char,
                        days[1],
                        op[1] as char,
                        strat[2] as char,
                        days[2]
                    )
                    .bytes()
                    .collect();
                    let fit = generation.problem.fitness(&g);

                    let g = Genotype::from(g, fit);

                    if g.fitness > best.front().unwrap().fitness {
                        best.pop_front();
                        best.push_back(g.clone());
                        pb.set_message(format!("${:.2}", best.back().unwrap().fitness.unwrap()));
                    }

                    if days[0] < 300 {
                        days[0] += 1;
                    } else if days[1] < 300 {
                        days[0] = 0;
                        days[1] += 1;
                    } else if days[2] < 300 {
                        days[0] = 0;
                        days[1] = 0;
                        days[2] += 1;
                    } else {
                        days[0] = 0;
                        days[1] = 0;
                        days[2] = 0;
                        break;
                    }

                    pb.inc(1);

                    if rx.try_recv().is_ok() {
                        println!();

                        for i in best.iter().rev() {
                            println!("{}", generation.problem.format(&i));
                        }
                    }
                }
            }
        }

        println!("Best Solution(s): ");

        for i in best.iter().rev() {
            println!("{}", generation.problem.format(&i));
        }
        */
        info!("Generation: 0 Best: {}", best.last().unwrap());

        pb.set_message(format!("${:.2}", best[0].fitness.unwrap()));

        for i in generation.population.iter() {
            debug!("\t{i}");
        }

        for i in 1..generation.max_generations {
            generation.generate_generation(i);

            pb.inc(1);

            let new = &generation.population[0];

            //pb.println(format!("{}", generation.problem.format(&new)));

            if new.fitness > best.first().unwrap().fitness {
                if best.len() == num {
                    best.remove(0);
                }

                best.push(new.clone());
                pb.set_message(format!("${:.2}", best.last().unwrap().fitness.unwrap()));
            }

            info!("Generation: {i} Best: {}", best.last().unwrap());

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

        println!("Best Solution(s): ");

        for i in best.iter().rev() {
            println!("{}", generation.problem.format(&i));
        }
    }
}
