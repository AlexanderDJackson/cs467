use std::fmt;
use clap::Parser;
use std::fs::File;
use std::cmp::Reverse;
use bitvec::prelude::*;
use std::io::{self, prelude::*, BufReader};

struct Knapsack {
    num_items: usize,
    weight: usize,
    items: Vec<Item>
}

#[derive(Clone)]
struct Item {
    id: String,
    value: usize,
    weight: usize
}

/// A program to find optimal knapsack solutions
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the file with the knapsack items
    #[arg(value_name = "FILE")]
    file: String,

    /// Three greedy algorithms: fast & worst
    #[arg(short, long)]
    greedy: bool,

    /// Exhaustive search algorithm: slow & best
    #[arg(short, long)]
    exhaustive: bool,

    /// Exhaustive search algorithm with pruning: faster & best
    #[arg(short, long)]
    better_exhaustive: bool,
}

impl Knapsack {
    fn add(&mut self, i: &Item) {
        self.items.push(i.clone());
        self.num_items = self.items.len();
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {} lbs, ${}", self.id, self.weight, self.value)
    }
}

impl fmt::Display for Knapsack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut sum: usize = 0;

        for item in self.items.clone() {
            sum += item.value;
        }

        match write!(
            f,
            "{} items, weight {} lbs, ${} total",
            self.num_items,
            self.weight,
            sum)
        {
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

fn parse_file(file_name: String) -> io::Result<Knapsack> {
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
            Err(e) => { panic!("Failed to parse line {}", e); }
        };

        let v: Vec<&str> = l.split(',').collect();

        if v.len() > 2 {
            items.items.push(
                Item {
                    id: String::from(v[0]),
                    value: match v[2].strip_prefix(" ").unwrap_or(v[2]).parse() {
                        Ok(s) => { s },
                        Err(e) => { panic!("Failed to parse value on line {}: {}", n, e); }
                    },
                    weight: match v[1].strip_prefix(" ").unwrap_or(v[1]).parse() {
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

fn increment(b: &mut BitVec::<u8>) {
    for bit in b.iter_mut().rev() {
        if !bit.clone() {
            bit.commit(true);
            break;
        } else {
            bit.commit(false);
        }
    }
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

fn exhaustive(k: &Knapsack) -> Knapsack {
    // I'm going to try and do some fancy stuff with bits
    // 15FEB23: Apparently not being able to handle more than 64 items
    //          is problematic :(
    let (mut max, mut max_weight, mut max_value) = (bitvec![u8, Lsb0; 0; k.items.len()], 0, 0);

    let mut bits = BitVec::with_capacity(k.items.len());
    bits.resize(k.items.len(), false);
            
    /* Iterate through the all numbers from 0 to num_states,
       use the bit values as the knapsack configuration
       15FEB23: I am now using the handy bitvec crate,
                which is a fancy wrapper around a vector
                of bits/bools, as my previous implementation
                used a usize, which is a 64 bit unsigned
                integer on 64 bit systems, and was thusly
                limited to knapsacks of 64 or fewer items.
    */
    while bits.contains(bits![0]) {
        // Add the items
        let (weight, value) = bits.iter_ones()
            .map(|i| (k.items[i].weight, k.items[i].value))
            .fold((0, 0), |(a, b), (w, v)| (a + w, b + v));

        // If we've found a new max within the max weight
        if weight <= k.weight && value > max_value {
            (max, max_weight, max_value) = (bits.clone(), weight, value);
        }

        increment(&mut bits);
    }

    Knapsack {
        num_items: max.iter_ones().count() as usize,
        weight: max_weight,
        items: max.iter_ones().map(|i| k.items[i].clone()).collect()
    }
}

fn exhaustive_pruning(k: &Knapsack) -> Knapsack {
    let mut bits = BitVec::with_capacity(k.items.len());
    bits.resize(k.items.len(), false);

    let (m, w, _) = recur(&k, (bits.clone(), 0, 0), bits, -1);

    Knapsack {
        num_items: m.count_ones() as usize,
        weight: w,
        items: m.iter_ones().map(|i| k.items[i].clone()).collect()
    }

}

// m: current max
// w: weight of max
// v: value of max
// n: number of items available to steal
// i: the bit that was just flipped,
//    we need to know this as we shouldn't
//    flip any bits to the right of this one,
//    otherwise we'll overflow the stack
fn recur<'a>(k: &Knapsack, (mut m, mut w, mut v): (BitVec::<u8>, usize, usize), mut b: BitVec::<u8>, i: isize) -> (BitVec::<u8>, usize, usize) {
    println!("{}", b);
    let (weight, value) = b.iter_ones()
        .map(|i| (k.items[i].weight, k.items[i].value))
        .fold((0, 0), |(a, b), (w, v)| (a + w, b + v));

    // Recur if under weight limit
    if weight <= k.weight {
        if value > v {
            (m, w, v) = (b.clone(), weight, value);
        } 
            
        for x in ((i + 1) as usize)..b.len() {
            if let Some(mut bit) = b.get_mut(x) { *bit ^= true; }
            (m, w, v) = recur(k, (m, w, v), b.clone(), x as isize);
        }
    }
    // We're over the weight limit, we can return and not recur,
    // thereby pruning this branch, or maybe we found a new max
    return (m, w, v);
}

fn main() {
    let args = Args::parse();
    
    let mut items: Knapsack;
    
    items = match parse_file(args.file) {
        Ok(i) => { i },
        Err(e) => { panic!("Failed to parse file: {}", e); }
    };

    println!("Knapsack of goods to be stolen:");
    println!("{}", items);

    if args.greedy {
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

    if args.better_exhaustive {
        println!("\nExhaustive search with pruning");
        println!("{}", exhaustive_pruning(&items));
    }

    if args.exhaustive {
        println!("\nExhaustive search");
        println!("{}", exhaustive(&items));
    }

    if !args.greedy && !args.exhaustive && !args.better_exhaustive {
        eprintln!("No method found, defaulting to exhaustive with pruning");
    }
}
