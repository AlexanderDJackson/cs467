use std::{fmt, time::{Duration, Instant}};
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
    /// Names of the files with the knapsack items
    #[arg(value_name = "FILE")]
    files: Vec<String>,

    /// Three greedy algorithms: fastest & worst
    #[arg(short, long)]
    greedy: bool,

    /// Exhaustive search algorithm: slow & best
    #[arg(short, long)]
    exhaustive: bool,

    /// Exhaustive search algorithm with pruning: faster & best
    #[arg(short, long)]
    better_exhaustive: bool,

    /// Hill climbing algorithm: Fastest & good
    #[arg(short, long)]
    climb: bool,

    /// Benchmarking mode: prints knapsack filename, time to run, value, weight, number of items in solution set
    #[arg(short, long)]
    time: bool,

    /// Time limit: apply a time limit in seconds to the algorithm, 0 = none
    #[arg(short, long, value_name = "SECONDS", default_value_t = 0)]
    limit: u64
}

impl Knapsack {
    fn add(&mut self, i: &Item) {
        self.items.push(i.clone());
        self.num_items = self.items.len();
    }
}

// Implement the Display trait to print our items
impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {} lbs, ${}", self.id, self.weight, self.value)
    }
}

// Implement the Display trait to print our knapsacks
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

// Read in the file and parse it into a knapsack
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

        // This is the first line with the number of items and the max weight
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
            items.weight = match v[1].strip_prefix(" ").unwrap_or(v[1]).parse() {
                Ok(s) => { s },
                Err(e) => { panic!("Failed to parse maximum weight on line {}: {}", n, e); }
            };
        }
    }

    Ok(items)
}

// Function to increment a bit vector
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

// Greedy algorithm, the type depends on how the list is sorted
fn greedy(k: &Knapsack) -> Knapsack {
    let mut stolen = Knapsack {
        num_items: 0,
        weight: 0,
        items: vec!()
    };

    for item in k.items.clone() {
        // If it fits, steal it
        if item.weight <= k.weight - stolen.weight {
            stolen.add(&item);
            stolen.weight += item.weight;
        }
    }

    stolen
}

fn exhaustive(k: &Knapsack, d: Option<Duration>) -> Knapsack {
    // I'm going to try and do some fancy stuff with bits
    // 15FEB23: Apparently not being able to handle more than 64 items
    //          is problematic :(
    let (mut max, mut max_weight, mut max_value) = (bitvec![u8, Lsb0; 0; k.items.len()], 0, 0);

    let mut bits = BitVec::with_capacity(k.items.len());
    bits.resize(k.items.len(), false);

    let before = Instant::now();
            
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

        if let Some(limit) = d {
            if before.elapsed() >= limit { break; }
        }
    }

    Knapsack {
        num_items: max.iter_ones().count() as usize,
        weight: max_weight,
        items: max.iter_ones().map(|i| k.items[i].clone()).collect()
    }
}

// Handler function for the recursive function
fn exhaustive_pruning(k: &Knapsack, d: Option<Duration>) -> Knapsack {
    let mut bits = BitVec::with_capacity(k.items.len());
    bits.resize(k.items.len(), false);

    let (m, w, _, _) = recur(
        &k,
        (bits.clone(), 0, 0, true),
        bits,
        -1,
        match d {
            None => { None },
            Some(d) => { Some((Instant::now(), d)) }
        }
    );

    Knapsack {
        num_items: m.count_ones() as usize,
        weight: w,
        items: m.iter_ones().map(|i| k.items[i].clone()).collect()
    }

}

// m: current max
// w: weight of max
// v: value of max
// cont: the flag to end searching if we've hit the time limit
// n: number of items available to steal
// i: the bit that was just flipped,
//    we need to know this as we shouldn't
//    flip any bits to the right of this one,
//    otherwise we'll overflow the stack
fn recur(
    k: &Knapsack,
    (mut m, mut w, mut v, mut cont): (BitVec::<u8>, usize, usize, bool),
    b: BitVec::<u8>,
    i: isize,
    d: Option<(Instant, Duration)>
) -> (BitVec::<u8>, usize, usize, bool) {

    if let Some((time, limit)) = d {
        if time.elapsed() >= limit {
            output(
                true,
                &String::from("filename"),
                &String::from("better_exhaustive"), 
                Some(time.elapsed()),
                &Knapsack {
                    num_items: m.count_ones() as usize,
                    weight: w,
                    items: m.iter_ones().map(|i| k.items[i].clone()).collect()
                }
            );
            panic!("guh");
            return (m, w, v, false);
        }
    }
    let (weight, value) = b.iter_ones()
        .map(|i| (k.items[i].weight, k.items[i].value))
        .fold((0, 0), |(a, b), (w, v)| (a + w, b + v));

    // Recur if under weight limit
    if weight <= k.weight {
        if value > v {
            (m, w, v) = (b.clone(), weight, value);
        } 
            
        // Swap bits to the right of x to prevent repeating
        for x in ((i + 1) as usize)..b.len() {
            let mut clone = b.clone();
            if let Some(mut bit) = clone.get_mut(x) { *bit ^= true; }

            (m, w, v, cont) = recur(k, (m, w, v, cont), clone, x as isize, d);

            if !cont {
                return (m, w, v, cont);
            }
        }
    }

    // We're over the weight limit, we can return and not recur,
    // thereby pruning this branch, or maybe we found a new max
    return (m, w, v, cont);
}

fn get_weight(bits: &BitVec<u8>, k: &Knapsack) -> usize {
    // Throwback to SML
    bits.iter_ones()
        .map(|i| k.items[i].weight)
        .fold(0, |acc, x| acc + x)
}

#[allow(dead_code)]
fn get_value(bits: &BitVec<u8>, k: &Knapsack) -> usize {
    bits.iter_ones()
        .map(|i| k.items[i].value)
        .fold(0, |acc, x| acc + x)
}

fn get_wv(bits: &BitVec<u8>, k: &Knapsack) -> (usize, usize) {
    bits.iter_ones()
        .map(|i| (k.items[i].weight, k.items[i].value))
        .fold((0, 0), |(a, b), (w, v)| (a + w, b + v))
}

// Repeatedly find the best elements to swap,
// return when there's nothing better to swap
fn swap(b: &BitVec<u8>, k: &Knapsack) -> BitVec<u8> {
    let mut bits = b.clone();

    loop {
        let (mut o, mut z): (isize, isize) = (-1, -1);
        let (w, v) = get_wv(&bits, &k);

        for one in bits.iter_ones() {
            for zero in bits.iter_zeros() {
                // The weight of all elements minus the one we're swapping out,
                // plus the one we're swapping in
                let test_weight = w - k.items[one].weight + k.items[zero].weight;
                // Same for values
                let test_value = v - k.items[one].value + k.items[zero].value;

                if test_weight <= k.weight && test_value > v
                && (z == -1 || test_weight <= w - k.items[one].weight + k.items[z as usize].weight)
                && (z == -1 || test_value > v - k.items[one].value + k.items[z as usize].value) {
                    (o, z) = (one as isize, zero as isize);
                }
            }
        }

        if let Some(mut one) = bits.get_mut(o as usize) { *one ^= true; }
        if let Some(mut zero) = bits.get_mut(z as usize) { *zero ^= true; };

        if z == -1 && o == -1 { break; }
    }

    bits
}

// Repeatedly find the best element to grab until nothing else fits
fn add(b: &BitVec<u8>, k: &Knapsack) -> BitVec<u8> {
    let mut bits = b.clone();

    loop {
        let w = get_weight(&bits, &k);

        if w == k.weight { break; }

        let mut z: isize = -1;

        for zero in bits.iter_zeros() {
            if w + k.items[zero].weight <= k.weight
            && (z == -1 || k.items[zero].value > k.items[z as usize].value) {
                z = zero as isize;
            }
        }

        if let Some(mut one) = bits.get_mut(z as usize) { *one ^= true; }

        if z == -1 { break; }
    }

    bits
}

// Repeatedly remove the best element until we're at the weight limit
fn sub(b: &BitVec<u8>, k: &Knapsack) -> BitVec<u8> {
    let mut bits = b.clone();

    loop {
        let w = get_weight(&bits, &k);

        if w <= k.weight { break; }

        let mut o: isize = -1;

        for one in bits.iter_ones() {
            if o == -1 || w - k.items[one].weight < w - k.items[o as usize].weight {
                o = one as isize;
            }
        }

        if let Some(mut one) = bits.get_mut(o as usize) { *one ^= true; }
    }

    bits
}

// The hill climbing algorithm
fn hill_climb(k: &Knapsack) -> Knapsack {
    let mut bits = BitVec::<u8>::with_capacity(k.items.len());
    bits.resize(k.items.len(), false);

    for mut bit in &mut bits {
        *bit = rand::random::<bool>();
    }

    let mut weight: usize;
    let before = Instant::now();

    for _ in 0..5 {

        if before.elapsed() >= Duration::from_secs(1200) { break; }

        weight = get_weight(&bits, &k);

        if weight < k.weight {
            bits = add(&bits, &k);
        } else if weight > k.weight {
            bits = sub(&bits, &k);
        } else {
            bits = swap(&bits, &k);
        }
    }

    Knapsack {
        num_items: bits.count_ones() as usize,
        weight: get_weight(&bits, &k),
        items: bits.iter_ones().map(|i| k.items[i].clone()).collect()
    }
}

// Function to determine print formatting based on commandline arguments
// a: benchmark argument
// f: file name
// m: algorithm used
// t: the runtime of the algorithm
// k: the resulting knapsack to print
fn output(a: bool, f: &String, m: &String, t: Option<Duration>, k: &Knapsack) {
    if a {
        println!("{},{},{:?},{},{},{}", f, m, t.unwrap().as_millis(), k.items.iter().fold(0, |acc, i| acc + i.value), k.weight, k.num_items);
    } else {
        println!("{k}\n");
    }
}

fn main() {
    let args = Args::parse();

    let mut knapsacks: Vec<(String, Knapsack)> = Vec::<(String, Knapsack)>::new();
    
    for file in args.files {
        knapsacks.push(
            (
                file.clone(), 
                match parse_file(file) {
                    Ok(i) => { i },
                    Err(e) => { panic!("Failed to parse file: {}", e); }
                }
            )
        );
    }

    benchmarking::warm_up();

    for (file, mut items) in knapsacks {
        if !args.time {
            println!("Knapsack of goods to be stolen:");
            println!("{}", items);
        }

        if args.greedy {
            let mut k: Vec<Knapsack> = Vec::<Knapsack>::new();

            // Sort by ascending weight
            let bench = benchmarking::measure_function_n(3, |m| {
                m[0].measure(|| {
                    items.items.sort_by_key(|x| x.weight);
                    k.push(greedy(&items));
                });

                // Sort by descending value
                m[1].measure(|| {
                    items.items.sort_by_key(|x| Reverse(x.value));
                    k.push(greedy(&items));
                });

                // Sort by descending weight:value ratio
                m[2].measure(|| {
                    items.items.sort_by_key(|x| x.weight / x.value);
                    k.push(greedy(&items));
                });
            }).unwrap();

            if !args.time { println!("\nGreedy solution prioritizing weight"); }
            output(args.time, &file, &String::from("greedy_weight"), Some(bench[0].elapsed()), &k[0]);
            
            if !args.time { println!("\nGreedy solution prioritizing value"); }
            output(args.time, &file, &String::from("greedy_value"), Some(bench[1].elapsed()), &k[1]);

            if !args.time { println!("\nGreedy solution prioritizing weight/value"); }
            output(args.time, &file, &String::from("greedy_ratio"), Some(bench[2].elapsed()), &k[2]);
        }

        if args.better_exhaustive {
            let mut k: Knapsack = Knapsack { num_items: 0, weight: 0, items: Vec::<Item>::new() };

            let bench = benchmarking::measure_function(|m| {
                m.measure(|| {
                    k = exhaustive_pruning(
                        &items,
                        match args.limit {
                            0 => { None },
                            n => { Some(Duration::from_secs(n)) }
                        }
                    )
                });
            }).unwrap();

            let m = String::from("exhaustive_pruning");

            if !args.time { println!("\nExhaustive search with pruning"); }
            output(args.time, &file, &m, Some(bench.elapsed()), &k);
        }

        if args.exhaustive {
            let mut k: Knapsack = Knapsack { num_items: 0, weight: 0, items: Vec::<Item>::new() };

            let bench = benchmarking::measure_function(|m| {
                m.measure(|| {
                    k = exhaustive(
                        &items,
                        match args.limit {
                            0 => { None },
                            n => { Some(Duration::from_secs(n)) }
                        }
                    )
                });
            }).unwrap();

            let m = String::from("exhaustive");

            if !args.time { println!("\nExhaustive search"); }
            output(args.time, &file, &m, Some(bench.elapsed()), &k);
        }

        if args.climb {
            let mut k: Knapsack = Knapsack { num_items: 0, weight: 0, items: Vec::<Item>::new() };

            let bench = benchmarking::measure_function(|m| {
                m.measure(|| {
                    k = hill_climb(&items)
                });
            }).unwrap();

            let m = String::from("hill_climb");

            if !args.time { println!("\nHill climbing"); }
            output(args.time, &file, &m, Some(bench.elapsed()), &k);
        }

        if !args.greedy && !args.exhaustive && !args.better_exhaustive && !args.climb {
            eprintln!("No method found");
        }
    }
}
