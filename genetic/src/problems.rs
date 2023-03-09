use crate::genetic::*;
use clap::ValueEnum;
use rand::Rng;
use std::fmt::{Display, Formatter, Result};

pub trait Problem {
    fn fitness(&self, genotype: &Vec<u8>) -> Fitness;
    fn mutate(&self, mutation_rate: f64, force_mutation: bool, genotype: &mut Genotype);
    fn generate_genotype(&self, force_create: bool) -> Genotype;
    fn format(&self, g: &Genotype) -> String;
    fn alphabet(&self) -> &Vec<u8>;
    fn len(&self) -> usize;
    fn new(files: Vec<String>) -> Option<Self>
    where
        Self: Sized;
}

#[derive(Clone, Debug, ValueEnum)]
pub enum ProblemType {
    Knapsack,
    Stocks,
}

impl Display for ProblemType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ProblemType::Knapsack => {
                write!(f, "knapsack")
            }
            ProblemType::Stocks => {
                write!(f, "stocks")
            }
        }
    }
}

pub mod knapsack {

    use crate::problems::*;
    use log::{debug, error, trace};
    use rand::thread_rng;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    pub struct Knapsack {
        pub alphabet: Vec<u8>,
        pub items: Vec<(usize, usize)>,
        pub max_weight: usize,
    }

    impl Problem for Knapsack {
        fn fitness(&self, g: &Vec<u8>) -> Fitness {
            let (weight, value) = self
                .items
                .iter()
                .zip(g.iter())
                .filter(|(_, b)| *b == &b'1')
                .fold((0, 0), |(weight, value), ((w, v), _)| {
                    (weight + w, value + v)
                });

            if weight > self.max_weight {
                Fitness::Invalid
            } else {
                let fitness = value as f64 / self.max_weight as f64;

                assert!(fitness.is_finite());
                assert!(fitness.is_sign_positive());

                // simple sigmoid function
                Fitness::Valid(fitness / (1.0 + fitness))
            }
        }

        fn format(&self, g: &Genotype) -> String {
            let (weight, value) = self
                .items
                .iter()
                .zip(g.genotype.iter())
                .filter(|(_, b)| *b == &b'1')
                .fold((0, 0), |(weight, value), ((w, v), _)| {
                    (weight + w, value + v)
                });

            format!(
                "weight: {}, value: {}, fitness: {}",
                weight,
                value,
                self.fitness(&g.genotype)
            )
        }

        fn generate_genotype(&self, force_create: bool) -> Genotype {
            let mut rng = thread_rng();
            let mut g = Genotype {
                genotype: Vec::<u8>::with_capacity(self.len()),
                fitness: Fitness::Invalid,
            };

            while g.len() < g.genotype.capacity() {
                g.genotype
                    .push(self.alphabet()[rng.gen_range(0..self.len())]);
            }

            trace!(
                "genotype: {:?}",
                g.genotype.iter().map(|b| *b as char).collect::<String>()
            );
            g.fitness = self.fitness(&g.genotype);

            if force_create {
                debug!("force creation enabled, mutating until valid...");
                while g.fitness == Fitness::Invalid {
                    self.mutate(0.1, true, &mut g);
                    trace!("{}", self.format(&g));
                }
            }

            debug!("created genitor: {g}");

            g
        }

        fn alphabet(&self) -> &Vec<u8> {
            &self.alphabet
        }

        fn len(&self) -> usize {
            self.items.len()
        }

        fn mutate(&self, mutation_rate: f64, force_mutation: bool, g: &mut Genotype) {
            let mut rng = rand::thread_rng();
            trace!("Force mutation: {}", force_mutation);

            trace!("Testing for mutations");
            for (n, c) in g.genotype.iter_mut().enumerate() {
                if rng.gen_bool(mutation_rate) {
                    trace!("Mutated gene {n} from: {}", *c as char);
                    if force_mutation {
                        let n = rng.gen_range(0..self.alphabet().len() - 1);
                        let m = self.alphabet()[n];
                        *c = if m == *c { self.alphabet()[n + 1] } else { m };
                    } else {
                        *c = self.alphabet()[rng.gen_range(0..self.alphabet().len())];
                    }

                    trace!("to: {}", *c as char);
                }
            }

            g.fitness = self.fitness(&g.genotype);
        }

        fn new(files: Vec<String>) -> Option<Knapsack> {
            let (max_weight, v) = parse_file(files[0].clone())?;

            Some(Knapsack {
                alphabet: vec![b'0', b'1'],
                items: v,
                max_weight,
            })
        }
    }

    fn parse_file(file_name: String) -> Option<(usize, Vec<(usize, usize)>)> {
        trace!("Reading {file_name}");
        let file = match File::open(&file_name) {
            Ok(f) => {
                debug!("Successfully read {}", file_name);
                f
            }
            Err(e) => {
                panic!("Error reading {}: {}", file_name, e);
            }
        };

        let reader = BufReader::new(file);
        trace!("Created BufReader for {file_name}");
        let mut v = Vec::<(usize, usize)>::new();
        trace!("Created vector for weight value pairs");
        let mut max_weight: usize = 0;
        trace!("max_weight = 0");

        for (n, line) in reader.lines().enumerate() {
            if let Ok(l) = line {
                trace!("Read line {n}: {l}");

                let temp: Vec<String> = l
                    .replace(" ", "") // remove spaces to yield CSV strings
                    .split(',') // split the string on the commas
                    .map(|l: &str| l.to_string()) // convert the &str's to owned String's
                    .collect(); // collect them as a vector

                trace!("Processed line {n}: {:?}", temp);

                if temp.len() > 2 {
                    if let Ok(x) = temp[1].parse::<usize>() {
                        if let Ok(y) = temp[2].parse::<usize>() {
                            trace!("Parsed {}: String as {x}: usize", temp[1]);
                            trace!("Parsed {}: String as {y}: usize", temp[2]);
                            v.push((x, y));
                            trace!("Pushed {:?} into v", (x, y));
                        } else {
                            error!("Failed to parse {}", temp[2]);
                        }
                    } else {
                        error!("Failed to parse {}", temp[1]);
                    }
                } else if temp.len() == 2 {
                    max_weight = match temp[1].parse::<usize>() {
                        Ok(num) => {
                            trace!("Parsed {}: String as {}: usize", temp[1], num);
                            trace!("max_weight = {num}");
                            num
                        }
                        Err(e) => {
                            panic!("Failed to get weight from {file_name}: {e}");
                        }
                    }
                }
            } else {
                error!("Failed to read line {}", n);
                return None;
            }
        }

        if max_weight == 0 {
            panic!("Failed to get weight from {file_name}");
        }

        trace!("Returning Some({:?})", (max_weight, &v));
        Some((max_weight, v))
    }
}

pub mod stocks {
    use log::{debug, trace};
    use rand::{thread_rng, Rng};

    use crate::genetic::{Fitness, Genotype};
    use crate::Problem;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    #[derive(Clone, Copy)]
    enum Average {
        Simple(usize),
        Exponential(usize),
        Maximum(usize),
    }

    pub struct Market {
        pub alphabet: Vec<u8>,
        pub histories: Vec<Vec<f64>>,
        pub funds: f64,
    }

    struct Actor {
        capital: f64,
        gains: f64,
        stocks: usize,
        strategy: (Average, char, Average, char, Average),
    }

    impl Average {
        fn unwrap(&self) -> usize {
            match self {
                Average::Simple(x) => *x,
                Average::Exponential(x) => *x,
                Average::Maximum(x) => *x,
            }
        }
    }

    impl Market {
        fn parse(chunk: [u8; 4]) -> Average {
            let days = ((chunk[1] as char).to_digit(10).expect("Invalid genotype!") * 100
                + (chunk[2] as char).to_digit(10).expect("Invalid genotype!") * 10
                + (chunk[3] as char).to_digit(10).expect("Invalid genotype!"))
                as usize;

            match chunk[0] {
                b's' => Average::Simple(days),
                b'e' => Average::Exponential(days),
                b'm' => Average::Maximum(days),
                _ => panic!("Invalid average type"),
            }
        }

        fn get_average(stock: &Vec<f64>, day: usize, average: &Average) -> f64 {
            match average {
                Average::Simple(days) => {
                    if day < *days || day == 0 {
                        0.0
                    } else {
                        stock[(day - days)..day].iter().sum::<f64>() / day as f64
                    }
                }
                Average::Exponential(days) => {
                    if day < *days || day == 0 {
                        0.0
                    } else {
                        let a = 2.0 / (*days as f64 + 1.0);
                        let (n, d, _) = stock[(day - days)..day].iter().fold(
                            (0.0, 0.0, 0.0),
                            |(n, d, x), p| {
                                (n + (p * (1.0 - a).powf(x)), d + (1.0 - a).powf(x), x + 1.0)
                            },
                        );

                        if d == 0.0 {
                            0.0
                        } else {
                            n / d
                        }
                    }
                }
                Average::Maximum(days) => {
                    if day < *days || day == 0 {
                        0.0
                    } else {
                        *stock[(day - *days)..day]
                            .iter()
                            .max_by(|a, b| a.partial_cmp(b).unwrap())
                            .unwrap_or(&0.0)
                    }
                }
            }
        }

        fn buy(actor: &mut Actor, price: f64) {
            if actor.capital < 2000.0 {
                if actor.gains > 2000.0 {
                    actor.gains -= 2000.0;
                    actor.capital += 2000.0;
                    debug!("Borrowing $2000 from gains");
                } else if actor.gains > 0.0 {
                    debug!("Borrowing ${} from gains", actor.gains);
                    actor.capital += actor.gains;
                    actor.gains = 0.0;
                }
            }

            if actor.capital < price {
                return;
            }

            let shares = (actor.capital / price) as usize;
            actor.capital -= shares as f64 * price;
            actor.stocks += shares;
            debug!(
                "Purchased {shares} stocks at ${price} a share to lose ${:.2}",
                shares as f64 * price
            );
            trace!(
                "{} shares, ${:.2} in capital, ${:.2} in gains",
                actor.stocks,
                actor.capital,
                actor.gains
            );
        }

        fn sell(actor: &mut Actor, price: f64) {
            if actor.stocks == 0 {
                return;
            }

            let shares = actor.stocks;
            actor.gains += shares as f64 * price;
            actor.stocks = 0;
            debug!(
                "Sold {shares} shares at ${price} to gain ${:.2}",
                shares as f64 * price
            );
            trace!(
                "{} shares, ${:.2} in capital, ${:.2} in gains",
                actor.stocks,
                actor.capital,
                actor.gains
            );
        }
    }

    impl Problem for Market {
        fn fitness(&self, genotype: &Vec<u8>) -> Fitness {
            let mut funds = 0.0;
            let strategy = (
                Market::parse(genotype[0..4].try_into().expect("Invalid genotype!")),
                genotype[4] as char,
                Market::parse(genotype[5..9].try_into().expect("Invalid genotype!")),
                genotype[9] as char,
                Market::parse(genotype[10..14].try_into().expect("Invalid genotype!")),
            );
            let days = (
                strategy.0.unwrap(),
                strategy.2.unwrap(),
                strategy.4.unwrap(),
            );
            let lowest = days.0.min(days.1).min(days.2);

            if days.0.max(days.1).max(days.2) == 0 {
                return Fitness::Valid(0.0);
            }

            for stock in &self.histories {
                let mut actor = Actor {
                    capital: self.funds,
                    gains: 0.0,
                    stocks: 0,
                    strategy,
                };

                for day in lowest..stock.len() - 1 {
                    let mut data = (false, false, false);

                    data.0 = if day < days.0 {
                        false
                    } else {
                        let avg = Market::get_average(&stock, day, &actor.strategy.0);

                        if avg == 0.0 {
                            if actor.strategy.1 == '&' {
                                true
                            } else {
                                false
                            }
                        } else {
                            avg < stock[day]
                        }
                    };

                    // true && ?
                    // implicit false && ?
                    if actor.strategy.1 == '&' && data.0 {
                        let avg = Market::get_average(&stock, day, &actor.strategy.2);
                        data.1 = if day < days.1 {
                            false
                        } else {
                            avg < stock[day]
                        };
                    // ? || ?
                    } else if actor.strategy.1 == '|' {
                        if data.0 {
                            data.1 = true;
                        } else {
                            let avg = Market::get_average(&stock, day, &actor.strategy.2);
                            data.1 = if day < days.1 || avg == 0.0 {
                                false
                            } else {
                                avg < stock[day]
                            };
                        }
                    }

                    // at this point, we already have the first two conditions evaluated
                    // the answer is in data.2, as we evaluate from left to right
                    // ? ** ? || ?
                    if actor.strategy.3 == '|' {
                        if data.1 {
                            data.2 = true;
                        } else {
                            let avg = Market::get_average(&stock, day, &actor.strategy.4);
                            data.2 = if day < days.2 || avg == 0.0 {
                                false
                            } else {
                                avg < stock[day]
                            };
                        }
                    // ? ** true && ?
                    // implicit ? ** false && ?
                    } else if actor.strategy.3 == '&' && data.1 {
                        let avg = Market::get_average(&stock, day, &actor.strategy.4);
                        data.2 = if day < days.2 {
                            false
                        } else {
                            avg < stock[day]
                        };
                    }

                    if data.0 == data.1 && data.1 == data.2 {
                        if data.0 {
                            Market::buy(&mut actor, stock[day]);
                        } else if days.0 > day || days.1 > day || days.2 > day {
                            Market::sell(&mut actor, stock[day]);
                        }
                    }
                }

                Market::sell(&mut actor, stock[stock.len() - 1]);
                debug!("Made ${:.2}", actor.gains + actor.capital);
                funds += actor.gains + actor.capital - self.funds;
            }

            let mut avg = funds / self.histories.len() as f64;
            debug!("Average return: ${:.2}", avg);

            // Simple sigmoid function
            avg = avg.clamp(0.0, f64::MAX);
            Fitness::Valid(avg / (avg + 1.0))
        }

        fn mutate(&self, mutation_rate: f64, force_mutation: bool, g: &mut Genotype) {
            let mut rng = rand::thread_rng();
            let methods = [b's', b'e', b'm'];
            let operators = [b'&', b'|'];

            let mut mutated = false;

            for i in 0..g.len() {
                if rng.gen_bool(mutation_rate) {
                    mutated = true;

                    match i {
                        0 | 5 | 10 => {
                            if force_mutation {
                                let new = rng.gen_range(0..2);

                                if g.genotype[i] == methods[new] {
                                    g.genotype[i] = methods[new + 1];
                                } else {
                                    g.genotype[i] = methods[new];
                                }
                            } else {
                                g.genotype[i] = methods[rng.gen_range(0..=2)];
                            }
                        }
                        1..=3 | 6..=8 | 11..=13 => {
                            if force_mutation {
                                let new = b'0' + rng.gen_range(0..9) as u8;

                                if g.genotype[i] == new {
                                    g.genotype[i] = new + 1;
                                } else {
                                    g.genotype[i] = new;
                                }
                            } else {
                                g.genotype[i] = b'0' + rng.gen_range(0..=9) as u8;
                            }
                        }
                        4 | 9 => {
                            if force_mutation {
                                if g.genotype[i] == b'&' {
                                    g.genotype[i] = b'|';
                                } else {
                                    g.genotype[i] = b'&';
                                }
                            } else {
                                g.genotype[i] = operators[rng.gen_range(0..=1)];
                            }
                        }
                        _ => {
                            panic!("Invalid genotype!");
                        }
                    }
                }
            }

            if mutated {
                g.fitness = self.fitness(&g.genotype);
            }
        }

        fn generate_genotype(&self, force_create: bool) -> Genotype {
            let mut rng = thread_rng();
            let mut g = Genotype {
                genotype: Vec::<u8>::with_capacity(self.len()),
                fitness: Fitness::Invalid,
            };

            let methods = [b's', b'e', b'm'];
            let operators = [b'&', b'|'];

            for i in 0..self.len() {
                match i {
                    0 | 5 | 10 => {
                        g.genotype.push(methods[rng.gen_range(0..=2)]);
                    }
                    1 | 2 | 6 | 7 | 11 | 12 => {
                        g.genotype.push(b'0' + rng.gen_range(0..2) as u8);
                    }
                    2..=3 | 7..=8 | 12..=13 => {
                        g.genotype.push(b'0' + rng.gen_range(0..=9) as u8);
                    }
                    4 | 9 => {
                        g.genotype.push(operators[rng.gen_range(0..=1)]);
                    }
                    _ => {
                        panic!("Invalid genotype!");
                    }
                }
            }

            trace!(
                "genotype: {:?}",
                g.genotype.iter().map(|b| *b as char).collect::<String>()
            );
            g.fitness = self.fitness(&g.genotype);

            if force_create {
                debug!("force creation enabled, mutating until valid...");
                while g.fitness == Fitness::Invalid {
                    self.mutate(0.1, true, &mut g);
                    trace!("{}", self.format(&g));
                }
            }

            debug!("created genitor: {g}");

            g
        }

        fn format(&self, g: &Genotype) -> String {
            if let Fitness::Valid(f) = self.fitness(&g.genotype) {
                if f == 0.0 {
                    return format!(
                        "{} ({}) lost money",
                        g.genotype.iter().map(|b| *b as char).collect::<String>(),
                        f
                    );
                } else {
                    return format!(
                        "{} ({}) made {:.2} dollars",
                        g.genotype.iter().map(|b| *b as char).collect::<String>(),
                        f,
                        -f / (f - 1.0)
                    );
                }
            } else {
                panic!("Invalid fitness!");
            }
        }

        fn alphabet(&self) -> &Vec<u8> {
            &self.alphabet
        }

        fn len(&self) -> usize {
            14
        }

        fn new(files: Vec<String>) -> Option<Self>
        where
            Self: Sized,
        {
            let mut histories = Vec::<Vec<f64>>::with_capacity(files.len());

            for (i, file) in files.iter().enumerate() {
                let file = match File::open(file) {
                    Ok(f) => {
                        histories.push(Vec::with_capacity(250));
                        f
                    }
                    Err(e) => {
                        panic!("Error reading {}: {}", file, e);
                    }
                };

                let reader = BufReader::new(file);

                for line in reader.lines() {
                    if let Ok(l) = line {
                        if l.contains("-") {
                            continue;
                        } else {
                            histories[i].push(l.parse::<f64>().expect("Invalid file!"));
                        }
                    }
                }
            }

            Some(Market {
                alphabet: vec![
                    b'&', b'|', b's', b'e', b'm', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7',
                    b'8', b'9',
                ],
                funds: 20000.0,
                histories,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{problems::Problem, stocks::Market, Fitness, genetic::Genotype};

    #[test]
    fn test_market_simple_moving_average() {
        let mut market = Market::new(vec!["testdata/tests/.txt".to_string()]).unwrap();
        let genotype = vec![ b's', b'0', b'0', b'0', b'&', b's', b'0', b'0', b'0', b'|', b's', b'0', b'0', b'0'];
        let fitness = Market::fitness(&market, &genotype);

        let g = Genotype::from(
            genotype,
            fitness,
        );
    }
}
