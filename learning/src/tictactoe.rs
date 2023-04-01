use crate::games::{Game, Player, Status};
use log::{debug, trace};
use std::{
    collections::HashMap,
    fmt::{Display, Formatter, Result},
};
use colored::*;

#[derive(Debug, Copy, Clone)]
pub struct TicTacToe {
    pub board: [u8; 9],
    pub next: char,
}

impl TicTacToe {
    pub fn new() -> TicTacToe {
        TicTacToe {
            board: [b'_'; 9],
            next: 'X',
        }
    }

    pub fn from(board: [u8; 9]) -> TicTacToe {
        TicTacToe {
            board,
            next: if board.iter().filter(|&&x| x == b'_').count() % 2 == 0 {
                'O'
            } else {
                'X'
            },
        }
    }
}

impl Game for TicTacToe {
    fn evaluate(&self) -> Status {
        // Check rows
        for row in 0..3 {
            if self.board[0 + row * 3] == self.board[1 + row * 3]
                && self.board[1 + row * 3] == self.board[2 + row * 3]
                && self.board[0 + row * 3] != b'_'
            {
                return Status::Win(self.board[0 + row * 3] as char);
            }
        }

        // Check columns
        for i in 0..3 {
            if self.board[i] == self.board[i + 3]
                && self.board[i + 3] == self.board[i + 6]
                && self.board[i] != b'_'
            {
                return Status::Win(self.board[i] as char);
            }
        }

        // Check diagonals
        if ((self.board[0] == self.board[4] && self.board[4] == self.board[8])
            || (self.board[2] == self.board[4] && self.board[4] == self.board[6]))
            && self.board[4] != b'_'
        {
            return Status::Win(self.board[4] as char);
        }

        // Check for draw
        for i in 0..9 {
            if self.board[i] == b'_' {
                return Status::Playing;
            }
        }

        Status::Draw
    }

    fn moves(&self) -> Option<Vec<(usize, Self)>> {
        let mut moves = Vec::new();

        match self.evaluate() {
            Status::Win(_) | Status::Draw => return None,
            _ => {
                for i in 0..9 {
                    if self.board[i] == b'_' {
                        let mut new_board = self.board.clone();
                        new_board[i] = self.next as u8;
                        moves.push((i, TicTacToe::from(new_board)));
                    }
                }

                Some(moves)
            }
        }
    }

    fn play(
        &mut self,
        map: &mut HashMap<String, [f64; 9]>,
        player1: &impl Player,
        player2: &impl Player,
    ) -> Status {
        // (board, number of moves, move made)
        let mut history = Vec::<(String, usize, usize)>::new();

        loop {
            let input = match self.next {
                'X' => player1.play(self, map),
                'O' => player2.play(self, map),
                _ => panic!("Invalid player"),
            };

            debug!("{} Turn", self.next);

            if let Some(input) = input {
                let index = input.parse::<usize>().unwrap();
                debug!("{} plays at {}", self.next, index);

                if index < self.board.len() && self.board[index] == b'_' {
                    self.board[index] = self.next as u8;
                    self.next = if self.next == 'X' { 'O' } else { 'X' };
                    history.push((
                        self.format(),
                        self.moves()
                            .unwrap_or(Vec::<(usize, TicTacToe)>::new())
                            .len(),
                        index,
                    ));

                    debug!("Pushed {} to history", self.format());
                }
            }

            match self.evaluate() {
                Status::Win(c) => {
                    let loser = if c == 'X' { 'O' } else { 'X' };
                    debug!("{c} wins!");
                    // Adjust probablities
                    for n in 1..history.len() {
                        let (new, _, num) = &history[history.len() - n];
                        let board = &history[history.len() - n - 1].0;
                        let num = *num;
                        let probs = map.get_mut(board).expect("Board not found in map");

                        debug!("Before: {} -> {} = {:?}", board, new, probs);
                        let probs = probs
                            .iter_mut()
                            .enumerate()
                            .collect::<Vec<(usize, &mut f64)>>();

                        let diff = if n % 2 == 1 {
                            *probs[num].1 * (1.0 + (1.0 / (n + 1) as f64)) - *probs[num].1
                        } else {
                            *probs[num].1 / (1.0 + (1.0 / (n + 1) as f64)) - *probs[num].1
                        };

                        if n % 2 == 1 {
                            trace!(
                                "Rewarding {c}: Winning move: {}, {} -> {}",
                                num,
                                probs[num].1,
                                *probs[num].1 + diff
                            );
                        } else {
                            trace!(
                                "Punishing {loser}: Losing move: {}, {} -> {}",
                                num,
                                probs[num].1,
                                *probs[num].1 + diff
                            );
                        }

                        for (j, p) in probs {
                            if j == num {
                                *p += diff;
                            } else {
                                trace!("\t{}: {} -> {}", j, p, *p + diff * *p);
                                *p += diff * *p;
                            }

                            *p = p.clamp(0.0, 1.0);
                            assert!(p.is_finite());
                        }

                        debug!(
                            "After: {} -> {} = {:?}",
                            board,
                            &history[history.len() - n - 1].0,
                            map.get(board).expect("Board not found in map")
                        );
                    }

                    return Status::Win(c);
                }
                Status::Draw => return Status::Draw,
                Status::Playing => continue,
            }
        }
    }

    fn format(&self) -> String {
        self.board.iter().map(|&x| x as char).collect::<String>()
    }
}

impl Display for TicTacToe {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut s = String::new();
        let chars = self
            .board
            .iter()
            .enumerate()
            .map(|(n, c)| if *c == b'_' { ((b'0' + n as u8) as char).to_string().italic().dimmed() } else { (*c as char).to_string().bold() })
            .collect::<Vec<ColoredString>>();

        for i in 0..2 {
            s.push_str(
                format!(
                    " {} | {} | {} \n",
                    chars[0 + i * 3],
                    chars[1 + i * 3],
                    chars[2 + i * 3]
                )
                .as_str(),
            );
            s.push_str(format!("---|---|---\n").as_str());
        }

        s.push_str(
            format!(
                " {} | {} | {} ",
                chars[6], chars[7], chars[8]
            )
            .as_str(),
        );

        s.replace("_", " ").fmt(f)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
