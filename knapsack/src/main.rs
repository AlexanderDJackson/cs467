use std::env;
use std::fmt;
use std::fs::File;
use std::cmp::Reverse;
use indicatif::{ProgressBar, ProgressIterator};
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
    let (mut max, mut max_weight, mut max_value) = (0, 0, 0);

    // Get the total of all possible knapsack states
    let num_states = 2_usize.pow(k.items.len() as u32);

    let (mut weight, mut value) = (0, 0);

    // Iterate through the all numbers from 0 to num_states,
    // use the bit values as the knapsack configuration
    for i in (0..num_states).progress() {
        let mut temp = i;
        
        (weight, value) = (0, 0);
            
        while temp != 0 {
            // Each 1 is an item in the knapsack
            let index = temp.trailing_zeros() as usize;
            let item = &k.items[index];

            // Add the item if possible
            if weight + item.weight <= k.weight {
                weight += item.weight;
                value += item.value;
            }

            // Set the least significant bit to 0 to find the next 1
            temp ^= 2_usize.pow(index as u32);
        }

        // If we've found a new max
        if value > max_value {
            (max, max_weight, max_value) = (i, weight, value);
        }
    }

    let mut knap = Knapsack {
        num_items: max.count_ones() as usize,
        weight: max_weight,
        items: vec!()
    };

    // Add our items to the knapsack
    while max != 0 {
        let index = max.trailing_zeros() as usize;
        knap.add(&k.items[index]);
        max ^= 2_usize.pow(index as u32);
    }

    knap
}

fn exhaustive_pruning(k: &Knapsack) -> Knapsack {
    let b = ProgressBar::new(2_u64.pow(k.items.len() as u32));

    let (mut m, w, _) = recur(&k, (0, 0, 0), 0, k.items.len(), -1, b.clone());

    let mut knap = Knapsack {
        num_items: m.count_ones() as usize,
        weight: w,
        items: vec!()
    };

    // Add our items to the knapsack
    while m != 0 {
        let index = m.trailing_zeros() as usize;
        knap.add(&k.items[index]);
        m ^= 2_usize.pow(index as u32);
    }

    knap
}

// m: current max
// w: weight of max
// v: value of max
// c: current configuration to test
// n: number of items available to steal
fn recur(k: &Knapsack, (mut m, mut w, mut v): (usize, usize, usize), c: usize, n: usize, i: i8, b: ProgressBar) -> (usize, usize, usize) {
    let mut temp = c;
    let mut weight = 0;
    let mut value = 0;

    while temp != 0 {
        // Each 1 is an item in the knapsack
        let index = temp.trailing_zeros() as usize;
        let item = &k.items[index];

        // Add the item if possible
        weight += item.weight;
        value += item.value;

        // Set the least significant bit to 0 to find the next 1
        temp ^= 2_usize.pow(index as u32);
    }

    if weight <= k.weight {
        if value > v {
            (m, w, v) = (c, weight, value);
        } 
            
        b.inc(1);

        for x in ((i + 1) as u32)..(n as u32) {
            (m, w, v) = recur(k, (m, w, v), c ^ (1usize << x), n, x as i8, b.clone());
        }
    } else {
        b.inc(2u64.pow((n - i as usize) as u32));
    }

    return (m, w, v);

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

    //println!("\nExhaustive search");
    //println!("{}", exhaustive(&items));

    println!("\nExhaustive search with pruning");
    println!("{}", exhaustive_pruning(&items));
}
