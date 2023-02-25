use crate::problems::knapsack::*;
use clap::Parser;
use genetic::*;
use simple_logger:: SimpleLogger;
use log::{trace, LevelFilter};

mod problems {
    pub mod knapsack;
}

fn main() {
    // Create logger that defaults to info level
    // Run with env variable RUST_LOG=<desired_level> to change from default
    match SimpleLogger::new().with_level(LevelFilter::Warn).env().init() {
        Ok(logger) => { logger },
        Err(e) => { eprintln!("Unable to intialize logger: {}", e); }
    }

    let args = Args::parse();

    trace!("Arguments: {:?}", args);

    let fitness;

    match args.problem {
        Problem::Knapsack => {
            let file_name = args.file.expect("file name expected!");
            let (max_weight, items) = match knapsack::parse_file(&file_name) {
                Some((m, i)) => { (m, i) },
                None => { panic!("Failed to parse {}", file_name); }
            };
                
            fitness = move |string: &String| -> (usize, usize, f64) { knapsack::fitness(items.clone(), max_weight, string) };
        }
    }
    
    let mut recent = args.genitors;
    let mut best = generate_genitors(
        &fitness,
        1,
        args.length,
        args.alphabet.chars().collect()
    )[0].to_string();

    for i in 0..args.max_generations {
        recent = generate_generation(
            &fitness,
            if recent.len() > 0 { Some(recent) } else { None },
            args.population,
            args.intermediate_population,
            args.length,
            args.alphabet.chars().collect::<Vec<char>>(),
            args.selection_method,
            args.sex_method,
            args.mutation_rate,
            args.skip,
            args.force_mutation,
            i
        );

        if fitness(&recent[0]).2 > fitness(&best).2 {
            best = recent[0].to_string();
        }
    }

    let (w, v, f) = fitness(&best);
    println!("Best Solution: {} (weight = {w}, value = {v}, fitness = {f})", best);
}
