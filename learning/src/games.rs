use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::{collections::HashMap, fmt::Display};

pub enum Status {
    Win(char),
    Draw,
    Playing,
}

pub struct Random;
pub struct Human;
pub struct AI;

pub trait Game: Copy + Display + Sized {
    fn evaluate(&self) -> Status;
    fn moves(&self) -> Option<Vec<(usize, Self)>>;
    fn play(&mut self, map: &mut HashMap<String, [f64; 9]>, player1: &impl Player, player2: &impl Player) -> Status;
    fn format(&self) -> String;
}

pub trait Player {
    fn play(&self, game: &impl Game, map: &mut HashMap<String, [f64; 9]>)
        -> Option<String>;
}

impl Player for Human {
    fn play(
        &self,
        game: &impl Game,
        _map: &mut HashMap<String, [f64; 9]>,
    ) -> Option<String> {
        println!("{game}");

        match game.evaluate() {
            Status::Win(c) => {
                println!("{c} wins!");
                return None;
            },
            Status::Draw => {
                println!("Draw!");
                return None;
            },
            Status::Playing => (),
        }

        let mut input = String::new();

        std::io::stdin().read_line(&mut input).unwrap();

        Some(input.trim().to_string())
    }
}

impl Player for AI {
    fn play(
        &self,
        game: &impl Game,
        map: &mut HashMap<String, [f64; 9]>,
    ) -> Option<String> {
        if let Some(moves) = game.moves() {
            let mut rng = rand::thread_rng();

            let probs = map.get(game.format().as_str()).expect(format!("Game not found: {}", game.format()).as_str());

            let dist = WeightedIndex::new(
                moves
                    .iter()
                    .map(|item| (item.1, probs[item.0]))
                    .collect::<Vec<_>>()
                    .iter()
                    .map(|item| item.1),
            )
            .unwrap();

            let choice = moves[dist.sample(&mut rng)].0.to_string();

            Some(choice)
        } else {
            None
        }
    }
}

impl Player for Random {
    fn play(
        &self,
        game: &impl Game,
        _map: &mut HashMap<String, [f64; 9]>,
    ) -> Option<String> {
        if let Some(moves) = game.moves() {
            let mut rng = rand::thread_rng();

            let choice = moves[rng.gen_range(0..moves.len())].0.to_string();

            Some(choice)
        } else {
            None
        }
    }
}
