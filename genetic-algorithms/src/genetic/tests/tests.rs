use genetic::*;
use rand::Rng;

// mutate((mut String, mut String), f64, Vec<char>, bool) -> (String, String)

// Ensure mutated genotypes are different
#[test]
fn mutate_works() {
    let child = (String::from("0123456789ABCDEF"), String::from("FEDCBA9876543210"));
    let mutated = 
        mutate(
            child.clone(),
            1.0,
            vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F'],
            true
        );
    
    assert_ne!(child, mutated, );
}

// Ensure non-mutated genotypes are not different
#[test]
fn mutate_probability_works() {
    let child = (String::from("0123456789ABCDEF"), String::from("FEDCBA9876543210"));
    let mutated = 
        mutate(
            child.clone(),
            0.0,
            vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F'],
            false
        );
    
    assert_eq!(child, mutated, );
}

// Ensure that if mutations are forced, all of
// the mutated genes have different allelles
#[test]
fn mutate_forced_mutations() {
    let child = (String::from("ABCD"), String::from("DCBA"));
    let mutated = 
        mutate(
            child.clone(),
            1.0,
            vec!['A', 'B', 'C', 'D'],
            true
        );

    for (old, new) in child.0.chars().zip(mutated.0.chars()) {
        assert_ne!(old, new);
    }

    for (old, new) in child.1.chars().zip(mutated.1.chars()) {
        assert_ne!(old, new);
    }
}

// Be mostly sure that if mutations are not forced, 
// not all of the mutated genes have different allelles
#[test]
fn mutate_unforced_mutations() {
    let mutated = 
        mutate(
            (
                String::from("AAAAAAAAAAAAAAAA"),
                String::from("BBBBBBBBBBBBBBBB")
            ),
            1.0,
            vec!['A', 'B'],
            false
        );

        assert!(mutated.0.contains('B'), "Holy moly, the chances of this failing are 0.000015%");
        assert!(mutated.1.contains('A'), "Holy moly, the chances of this failing are 0.000015%");
}

// reproduce((String, String), SexMethod, Vec<char>, f64, f64, bool) -> (String, String)

// Ensure that parents/genitors have a chance to carry over to the next generation
#[test]
fn reproduce_carry_over_parents() {
    let parent = (String::from("ABCD"), String::from("DCBA"));

    assert_eq!(
        parent,
        reproduce(
            parent.clone(),
            SexMethod::Uniform,
            vec!['A', 'B', 'C', 'D'],
            0.0,
            1.0,
            false
        )
    );
}

// Ensure resulting children share genes with parents
#[test]
fn reproduce_children_inherit_parent_genes() {
    let parent = (String::from("ABCD"), String::from("DCBA"));
    let child = 
        reproduce(
            parent.clone(),
            SexMethod::Uniform,
            vec!['A', 'B', 'C', 'D'],
            0.0,
            0.0,
            false
        );

    assert!(
        child.0.chars().zip(parent.0.chars().zip(parent.1.chars()))
            .fold(true, |same, (c, (p0, p1))| same && (c == p0 || c == p1))
    );

    assert!(
        child.1.chars().zip(parent.0.chars().zip(parent.1.chars()))
            .fold(true, |same, (c, (p0, p1))| same && (c == p0 || c == p1))
    );
}

// Ensure reproduction method (number of crossover points) is respected
#[test]
fn reproduce_one_crossover_point() {
    let parent = (String::from("0123456789ABCDEF"), String::from("FEDCBA9876543210"));
    let child = 
        reproduce(
            parent.clone(),
            SexMethod::One,
            vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F'],
            0.0,
            0.0,
            false
        );

    let mut test = child.0[0..1] == parent.0[0..1] && child.1[0..1] == parent.1[0..1];

    assert_eq!(
        child.0.chars().zip(child.1.chars())
            .zip(parent.0.chars().zip(parent.1.chars()))
            .fold(0, |num_points, ((c0, c1), (p0, p1))| {
                if test == (c0 == p0 && c1 == p1) {
                    num_points
                } else {
                    test = !test;
                    num_points + 1
                }
            }
        ),
        1
    );

}

// Ensure reproduction method (number of crossover points) is respected
#[test]
fn reproduce_two_crossover_point() {
    let parent = (String::from("0123456789ABCDEF"), String::from("FEDCBA9876543210"));
    let child = 
        reproduce(
            parent.clone(),
            SexMethod::Two,
            vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F'],
            0.0,
            0.0,
            false
        );

    let mut test = child.0[0..1] == parent.0[0..1] && child.1[0..1] == parent.1[0..1];

    assert_eq!(
        child.0.chars().zip(child.1.chars())
            .zip(parent.0.chars().zip(parent.1.chars()))
            .fold(0, |num_points, ((c0, c1), (p0, p1))| {
                if test == (c0 == p0 && c1 == p1) {
                    num_points
                } else {
                    test = !test;
                    num_points + 1
                }
            }
        ),
        2
    );

}

// Ensure reproduction method (number of crossover points) is respected
#[test]
fn reproduce_uniform_crossover_point() {
    let parent = (String::from("0123456789ABCDEF"), String::from("FEDCBA9876543210"));
    let child = 
        reproduce(
            parent.clone(),
            SexMethod::Uniform,
            vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F'],
            0.0,
            0.0,
            false
        );

    let mut test = child.0[0..1] == parent.0[0..1] && child.1[0..1] == parent.1[0..1];

    assert!(
        child.0.chars().zip(child.1.chars())
            .zip(parent.0.chars().zip(parent.1.chars()))
            .fold(0, |num_points, ((c0, c1), (p0, p1))| {
                if test == (c0 == p0 && c1 == p1) {
                    num_points
                } else {
                    test = !test;
                    num_points + 1
                }
            }
        ) > 2
    );
}

// generate_genitors(usize, usize, Vec<char>) -> Vec<String>

// Ensure the correct number of genitors are created
#[test]
fn generate_genitors_correct_population() {
    let population = rand::thread_rng().gen_range(10..200);
    assert_eq!(population, generate_genitors(population, 10, vec!['A', 'B', 'C', 'D']).len());
}

// Ensure the genitors have the correct alphabet
#[test]
fn generate_genitors_correct_alphabet() {
    let population = rand::thread_rng().gen_range(10..100);
    let alphabet = vec!['A', 'B', 'C', 'D'];
    assert!(
        generate_genitors(population, 10, alphabet.clone())
            .iter()
            .fold(true, |good, genitor| 
                good && genitor
                    .chars()
                    .fold(true, |g, c| g && alphabet.contains(&c)
                )
            )
    );
}

// Ensure the genitors have the correct length
#[test]
fn generate_genitors_correct_length() {
    let population = rand::thread_rng().gen_range(10..100);
    let length = 10;
    assert!(
        generate_genitors(population, length, vec!['A', 'B', 'C', 'D'])
            .iter()
            .fold(true, |good, genitor| good && genitor.len() == length)
    );
}

// generate_population(impl Fn(String) -> f64, Option<Vec<String>>, usize, usize, Vec<char>, SelectionMethod) -> Vec<String>

