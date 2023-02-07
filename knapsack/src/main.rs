use std::env;
use std::fmt;
use std::fs::File;
use std::cmp::Reverse;
use std::io::{self, prelude::*, BufReader};

struct Knapsack {
    num_items: u16,
    weight: u16,
    items: Vec<Item>
}

#[derive(Clone)]
struct Item {
    id: String,
    value: u16,
    weight: u16
}

impl Knapsack {
    fn add(&mut self, i: &Item) {
        self.num_items += 1;
        self.items.push(i.clone());
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {} lbs, ${}", self.id, self.weight, self.value)
    }
}

impl fmt::Display for Knapsack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut sum: u16 = 0;

        for item in self.items.clone() {
            sum += item.value;
        }

        match write!(f, "{} items, weight {} lbs, ${} total", self.num_items, self.weight, sum) {
            Ok(()) => {},
            Err(e) => {
                eprint!("Failed to print: {}", e);
            }
        }

        for item in self.items.clone() {
            match write!(f, "\n{}", item) {
                Ok(()) => {},
                Err(e) => {
                    eprint!("Failed to print: {}", e);
                }
            }
        }

        Ok(())
    }
}

fn parse_file(file_name: &str) -> io::Result<Knapsack> {
    let file = File::open(file_name)?;
    let reader = BufReader::new(file);

    let mut items = 
    Knapsack {
        num_items: 0,
        weight: 0,
        items: Vec::new()
    };

    for (n, line) in reader.lines().enumerate() {
        let l = match line {
            Ok(s) => { s },
            Err(e) => { panic!("Failed to parse line in {}: {}", file_name, e); }
        };

        let v: Vec<&str> = l.split(',').collect();

        if v.len() > 2 {
            items.items.push(
                Item {
                    id: String::from(v[0]),
                    value: match v[2][1..].parse() {
                        Ok(s) => { s },
                        Err(e) => { panic!("Failed to parse value on line {}: {}", n, e); }
                    },
                    weight: match v[1][1..].parse() {
                        Ok(s) => { s },
                        Err(e) => { panic!("Failed to parse weight on line {}: {}", n, e); }
                    },
                }
            );
        } else {
            items.num_items = match v[0].parse() {
                Ok(s) => { s },
                Err(e) => { panic!("Failed to parse number of items on line {}: {}", n, e); }
            };
            items.weight = match v[1][1..].parse() {
                Ok(s) => { s },
                Err(e) => { panic!("Failed to parse maximum weight on line {}: {}", n, e); }
            };
        }
    }

    Ok(items)
}

fn greedy(k: &Knapsack) -> Knapsack {
    let mut stolen = Knapsack {
        num_items: 0,
        weight: 0,
        items: vec!()
    };

    for item in k.items.clone() {
        if item.weight <= k.weight - stolen.weight {
            stolen.add(&item);
            stolen.weight += item.weight;
        }
    }

    stolen
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut items: Knapsack;
    
    if args.len() > 1 {
        items = match parse_file(&args[1]) {
            Ok(i) => { i },
            Err(e) => { panic!("Failed to parse {}: {}", &args[1], e); }
        };
    } else {
        eprintln!("No filename found!");
        return;
    }

    println!("Knapsack of goods to be stolen:");
    println!("{}", items);

    println!("\nGreedy solution prioritizing weight");
    // Sort by ascending weight
    items.items.sort_by_key(|x| x.weight);
    println!("{}", greedy(&items));

    println!("\nGreedy solution prioritizing value");
    // Sort by descending value
    items.items.sort_by_key(|x| Reverse(x.value));
    println!("{}", greedy(&items));

    println!("\nGreedy solution prioritizing weight/value");
    // Sort by descending weight:value ratio
    items.items.sort_by_key(|x| x.weight / x.value);
    println!("{}", greedy(&items));
}
