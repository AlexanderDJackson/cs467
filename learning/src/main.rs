mod games;
mod tictactoe;

use clap::Parser;
use indicatif::ProgressStyle;
use log::{error, info, warn, LevelFilter};
use simple_logger::SimpleLogger;
use std::collections::HashMap;
use std::path::PathBuf;

use games::{Game, Status};
use tictactoe::TicTacToe;

#[derive(Debug, Parser)]
#[command(author, about, version)]
struct Args {
    /// The number of games to train for
    #[clap(short, long, default_value = "1000")]
    train: usize,

    /// Whether to play against the AI
    #[clap(short, long, default_value = "false")]
    ai: bool,

    /// Whether to play against a human
    #[clap(short, long, default_value = "false")]
    human: bool,

    /// Whether to play against a random player
    #[clap(short, long, default_value = "false")]
    random: bool,

    /// The file to load the tree from
    #[clap(short, long)]
    load: Option<PathBuf>,

    /// The file to save the tree to
    #[clap(short, long)]
    save: Option<PathBuf>,

    /// Whether to show progress bar
    #[clap(short, long, default_value = "false")]
    progress: bool,
}

fn build_tree(game: TicTacToe, map: &mut HashMap<String, [f64; 9]>) {
    if let Some(moves) = game.moves() {
        let mut probs = [if moves.len() > 0 { 1.0 / moves.len() as f64 } else { 0.0 }; 9];

        for (j, _) in game
            .board
            .iter()
            .enumerate()
            .filter(|(_, &x)| x != b'_')
        {
            probs[j] = 0.0;
        }

        assert_eq!(probs.iter().filter(|&&x| x > 0.0).count(), moves.len());

        assert!(
            probs.iter().sum::<f64>() > 0.0 || !game.board.contains(&b'_'),
            "{} {:?} {:?}",
            1.0 / moves.len() as f64,
            probs,
            game.format()
        );

        map.insert(game.format(), probs);

        for (_, i) in &moves {
            build_tree(*i, map);
        }
    } else {
        map.insert(game.format(), [0.0; 9]);
    }
}

fn main() {
    let args = Args::parse();
    let board = TicTacToe::new(); //from(*b"XOXOOXOX_");
    let mut map = HashMap::<String, [f64; 9]>::new();
    SimpleLogger::new()
        .with_level(LevelFilter::Warn)
        .env()
        .init()
        .unwrap();

    if let Some(load) = args.load {
        info!("Loading tree from {}", load.display());
        map = serde_json::from_reader(std::fs::File::open(load).unwrap()).unwrap();
    } else {
        warn!("No load path provided, loading tree.json");

        if let Ok(file) = std::fs::File::open("tree.json") {
            map = serde_json::from_reader(file).expect("Failed to load tree.json");
        } else {
            warn!("Could not read tree.json, building tree from scratch");
            build_tree(board, &mut map);
        }
    }

    let pb = if args.progress {
        let bar = indicatif::ProgressBar::new(args.train as u64);
        bar.set_message("Draw rate: 0%");
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{msg} {wide_bar} {pos}/{len}")
                .expect("Failed to set progress bar style"),
        );

        bar
    } else {
        indicatif::ProgressBar::hidden()
    };

    for _ in 0..100 {
        let (mut wins, mut draws) = (0, 0);
        for _ in 0..(args.train / 100) {
            let result = if args.ai && args.human && !args.random {
                TicTacToe::new().play(&mut map, &games::Human, &games::AI)
            } else if args.ai && !args.human && !args.random {
                TicTacToe::new().play(&mut map, &games::AI, &games::AI)
            } else if args.ai && args.random && !args.human {
                TicTacToe::new().play(&mut map, &games::Random, &games::AI)
            } else if args.human && !args.random && !args.ai {
                TicTacToe::new().play(&mut map, &games::Human, &games::Human)
            } else if args.random && !args.human && !args.ai {
                warn!("Why lol");
                TicTacToe::new().play(&mut map, &games::Random, &games::Random)
            } else if args.random && args.human && !args.ai {
                TicTacToe::new().play(&mut map, &games::Human, &games::Random)
            } else {
                error!("No player specified");
                std::process::exit(1);
            };

            match result {
                Status::Win(_) => wins += 1,
                Status::Draw => draws += 1,
                _ => println!("Something went wrong"),
            };
            pb.inc(1);
        }
        pb.set_message(format!("Draw rate: {}%", draws * 100 / (wins + draws)));
    }

    if let Some(path) = args.save {
        serde_json::to_writer_pretty(std::fs::File::create(path).unwrap(), &map).unwrap();
    } else {
        warn!("No save path provided, defaulting to tree.json");
        serde_json::to_writer_pretty(std::fs::File::create("tree.json").unwrap(), &map)
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_tree() {
        let mut map = HashMap::<String, [f64; 9]>::new();
        build_tree(TicTacToe::new(), &mut map);
        assert_eq!(map.len(), 362880);
    }
}
