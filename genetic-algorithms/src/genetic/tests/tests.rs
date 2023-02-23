use genetic::*;

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
/*
#[test]
fn mutate_carry_over_parents() {
    let parent = (String::from("ABCD"), String::from("DCBA"));

    assert_eq!(
        parent,
        mutate(
            parent,
            1.0,
            vec!['A', 'B', 'C', 'D'],
            true
        )
    );
}
*/

// generate_genitors(usize, usize, Vec<char>) -> Vec<String>
// generate_population(impl Fn(String) -> f64, Option<Vec<String>>, usize, usize, Vec<char>, SelectionMethod) -> Vec<String>

