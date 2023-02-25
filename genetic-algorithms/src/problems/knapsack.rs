pub mod knapsack {

    use std::fs::File;
    use std::io::{BufReader, BufRead};
    use log::{trace, debug, error};

    pub fn average_fitness(items: Vec<(usize, usize)>, max_weight: usize) -> f64 {
        let (weight, value) = items                                   // get the vector of weight value pairs
            .iter()                             // get an iterator from that vector
            .fold(                              // sum together the weights and values
                (0, 0),                         // the initial values
                |(weight, value), (w, v)|       // for each iterator, take in:
                                                // the previous closure's return value as weight and value,
                                                // and w and v from the current iterator,
                (weight + w, value + v)         // and add those to the current weight and value sums
            );

        if weight > max_weight { -1.0 } else { (value as f64 / max_weight as f64) / items.len() as f64 }
    }

    pub fn fitness(items: Vec<(usize, usize)>, max_weight: usize, string: &String) -> (usize, usize, f64) {
        let (weight, value) = items
            .iter()
            .zip(string.chars())
            .filter(|(_, b)| b == &'1')
            .fold(
                (0, 0),
                |(weight, value), ((w, v), _)|
                (weight + w, value + v)
            );

        (
            weight,
            value,
            if weight > max_weight { -1.0 } else { value as f64 / max_weight as f64 }
        )
    }

    pub fn parse_file(file_name: &String) -> Option<(usize, Vec<(usize, usize)>)> {
        trace!("Reading {file_name}");
        let file = match File::open(&file_name) {
            Ok(f) => {
                debug!("Successfully read {}", file_name);
                f
            },
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
                    .replace(" ", "")               // remove spaces to yield CSV strings
                    .split(',')                     // split the string on the commas
                    .map(|l: &str| l.to_string())   // convert the &str's to owned String's
                    .collect();                     // collect them as a vector
                
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
                        },
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
