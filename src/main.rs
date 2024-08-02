#![allow(dead_code)]

/*
 * written by Pacôme Perrotin
 */

use indicatif::{ParallelProgressIterator, ProgressStyle};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::iter::Iterator;

use rand::Rng;

/*
 * This single file program computes checks the validity of our
 * sequential solution to the density classification tasks on all configurations
 * up to size 30.
 *
 * Install rust and compile with "cargo run --release" after having
 * uncommented the desired line in the main function's body.
 */


fn main() {

    // To show an execution from a random configuration, uncomment this line.
    // The parameter controls the size of the initial configuration.
    // show_random_execution(13);

    // To check the solution on all configurations from sizes 2 to 30,
    // uncomment the following line. Can take a while!
    search_all();
}

/**
 * This struct encodes the state of a configuration of sizes up to 31.
 * To allow for the best performances, we do not use any array types,
 * and instead encode the information in 32 bits numbers.
 * One number is used for each property we would like to keep track of.
 * This results in a very fast execution, even for configuration of size 30,
 * because most of the program's memory is likely to fit in a CPU cache.
 *
 * While the Configuration struct could theoretically be more compact
 * (when the "taken" flag is 1, the "value" flag becomes useless)
 * it would be at the detriment of speed.
 *
 * We always assume all intermediate values (all except size and value)
 * are at 0 at the start of an execution, otherwise the program would
 * lead to undefined behavior.
 */
#[derive(Default)]
pub struct Configuration {
    // How many bits do we use on each following number?
    pub size : u32,
    // Is the value a 0 or a 1?
    pub value : u32,
    // Is the current symbol from the intermediate alphabet?
    pub alphabet : u32,
    // Has the symbol been removed using an X?
    pub taken : u32,
    // Is the local counter odd or even?
    pub color : u32,
    // Does the local memory contain a 0?
    pub mem_0 : u32,
    // Does the local memory contain a 1?
    pub mem_1 : u32,
}

/**
 * A helper function which copies a flag from another in a u32 number.
 * Inlined for better performances.
 */
#[inline]
fn self_assign(mem : &mut u32, to_index : u32, from_index : u32) {
    if *mem & 1 << from_index != 0 {
        *mem |= 1 << to_index;
    }
    else {
        *mem &= !(1 << to_index);
    }
}

/**
 * A helper function which assigns a boolean value to a specific bit
 * of a u32 number. Inlined for better performances.
 */
#[inline]
fn assign_bool(to : &mut u32, to_index : u32, value : bool) {
    if value {
        *to |= 1 << to_index;
    }
    else {
        *to &= !(1 << to_index);
    }
}

impl Configuration {
    /**
     * Creates a new configuration of a given size and value.
     * Passing in a value with 1 bits beyond the given size leads to
     * undefined behavior.
     */
    pub fn new(value : u32, size : u32) -> Self {
        Self {
            size, value, ..Default::default()
        }
    }

    /**
     * Prints the configuration to the screen using three lines,
     * the first indicates the values of the configuration (or X
     * if it was taken), the second line indicates the value of
     * the local counter (R or B), and the third line indicates
     * the value of the local memory (_ for empty, , for {1}, .
     * for {0}, and ; for {0, 1}). If a symbol is not from the
     * intermediary alphabet, its spot on the second and third line
     * are left blank.
     */
    pub fn println(&self) {
        // first line
        for k in 0..self.size {
            if self.alphabet & 1 << k != 0 && self.taken & 1 << k != 0 {
                print!("X");
            }
            else if self.value & 1 << k != 0 {
                print!("1");
            }
            else {
                print!("0");
            }
        }
        println!();

        // second line
        for k in 0..self.size {
            if self.alphabet & 1 << k == 0 {
                print!(" ");
            }
            else if self.color & 1 << k != 0 {
                print!("R");
            }
            else {
                print!("B");
            }
        }
        println!();

        // third line
        for k in 0..self.size {
            if self.alphabet & 1 << k == 0 {
                print!(" ");
            }
            else {
                match (self.mem_0 & 1 << k != 0, self.mem_1 & 1 << k != 0) {
                    (false, false) => print!("_"),
                    (true, false) => print!("."),
                    (false, true) => print!(","),
                    (true, true) => print!(";"),
                }
            }
        }
        
        println!();
    }

    /**
     * The function which does the real work and applies the automata's
     * local function at a given index. As our local rule is
     * sequential, only one index is updated.
     * The left parameter is used to indicate which index is at the left
     * of the current value; this value depends on the size of the configuration
     * and passing it this way saves a step of computation.
     */
    #[inline]
    pub fn apply_local_function(&mut self, left : u32, index : u32) {
        let left_mask = 1 << left;
        let index_mask = 1 << index;

        // if left is boolean
        if self.alphabet & left_mask == 0 {
            // if we are boolean
            if self.alphabet & index_mask == 0 {
                // 00 -> 0, 11 -> 1
                if (self.value & left_mask == 0) == (self.value & index_mask == 0) {
                    return;
                }

                // 01 or 10, kick start 
                self.alphabet |= index_mask; // we are now intermediate
                if self.value & index_mask != 0 { // we put the character in memory
                    self.mem_1 |= index_mask;
                }
                else {
                    self.mem_0 |= index_mask;
                }
                self.taken |= index_mask; // and remove the character

                return;
            }

            // if we are not boolean, propagation
            self.alphabet &= ! index_mask; // we are now boolean
            self_assign(&mut self.value, index, left); // we copy the value from left

            return;
        }

        // left is intermediate
        
        // if we are boolean or not the same color
        if self.alphabet & index_mask == 0 ||
          (self.color & left_mask == 0) != (self.color & index_mask == 0) {
            // we are scanning, we propagate the color and update the memory
            
            self.alphabet |= index_mask; // we ensure we are intermediate
            self_assign(&mut self.color, index, left); // we copy the color

            self_assign(&mut self.mem_0, index, left); // we copy the memory
            self_assign(&mut self.mem_1, index, left);

            // character already taken, task finished
            if self.taken & index_mask != 0 {
                return;
            }

            let value = self.value & index_mask != 0;
            if ! value && self.mem_0 & index_mask != 0 { // value is 0 and we already have one
                return;
            }
            if value && self.mem_1 & index_mask != 0 { // value is 1 and we already have one
                return;
            }

            self.taken |= index_mask; // we take the character

            if ! value { // and update the memory
                self.mem_0 |= index_mask;
            }
            else {
                self.mem_1 |= index_mask;
            }

            return;
        }

        // we are the same color, we are the brain of the configuration
        
        // if left has a complete set in memory
        if self.mem_0 & left_mask != 0 && self.mem_1 & left_mask != 0 {
            let color = self.color & index_mask != 0;
            assign_bool(&mut self.color, index, ! color); // we invert the color
            self.mem_0 &= ! index_mask; // we reset the memory
            self.mem_1 &= ! index_mask;

            // we don't have to try to add the current character, because
            // it is always taken at the kickstart
            
            return;
        }

        // from here on, all cases are reverting to boolean for convergence

        self.alphabet &= ! index_mask; // we revert to boolean

        // density 1
        if self.mem_1 & left_mask != 0 {
            assign_bool(&mut self.value, index, true); // we set value to 1
            return;
        }
        
        // density 0 or failure
        assign_bool(&mut self.value, index, false); // we set value to 0

        // we default to all 0 on failure to allow for convergence detection
    }

    /**
     * Applies the local function on every index in order.
     * At this step, we can easily define what the "left" index
     * is and pass it to the apply_local_function method.
     */
    pub fn update(&mut self) {
        self.apply_local_function(self.size - 1, 0);

        for k in 1..self.size {
            self.apply_local_function(k - 1, k);
        }
    }

    /**
     * Returns true if the configuration contains no intermediary symbol
     * and that all the values are either 0 or 1.
     *
     * If the value passed to the new function contained 1 bits beyond the
     * defined size, this function will return false even if all the bits
     * within the size are equal.
     */
    pub fn has_converged(&self) -> bool {
        self.alphabet == 0 && // no intermediate symbols
        (self.value == 0 || self.value == (1 << self.size) - 1)
        // all values are 0 or all values are 1
    }

    /**
     * If our local rule fails to compute the correct density value for
     * the current configuration, this function returns false.
     * It does it by computing the real density value of the initial
     * configuration, and then runs the automata to check if the
     * two values are coherent.
     * If the initial configuration had as many 1s than 0s (in the case
     * of an even size), the function always returns true, as our
     * automata is then not expected to follow any particular behavior,
     * and is thus correct.
     */
    pub fn is_correct(&mut self) -> bool {
        let mut count_0 = 0;
        let mut count_1 = 0;
        for k in 0..self.size {
            if self.value & 1 << k == 0 { count_0 += 1; }
            else { count_1 += 1; }
        }

        if count_0 == count_1 { return true; } // in case of equality, undefined behavior

        let majority = if count_0 > count_1 { 0 }
            else { 1 };

        let mut iteration_count = 0;

        while ! self.has_converged() {

            if iteration_count > self.size { // We should take around size / 2
                return false;
            }

            self.update();
            iteration_count += 1;
        }

        majority == self.value & 1 // configuration is uniform, so we only test the first bit
    }
}

/**
 * This function iterates through all the configurations of a given size,
 * and returns any counter-example on which the is_correct method returns
 * false. If no counter example is found, it returns None instead.
 *
 * This function makes uses of parallel iterators for more speed.
 */
fn find_counter_example(size : u32) -> Option<u32> {
    let progress_style =
        ProgressStyle::with_template("[{eta}] {pos:10}/{len:10} {bar:40}").unwrap();

    (0..1 << size - 1)
        .into_par_iter()
        .progress_with_style(progress_style)
        .map(|k| (k, Configuration::new(k, size).is_correct()) )
        .filter(|(_, b)| ! b) // we keep the ones that failed
        .map(|(k, _)| k)
        .take_any(1)
        //.take(1)
        .collect::<Vec<_>>()
        .iter()
        .next()
        .copied() // and return the first one, if there is any
}

/**
 * Helper function which calls find_counter_example, and if a counter example
 * is found, prints a nice error about it, as well as the execution of
 * the counter example, for inspection by the user.
 */
fn search_size(size : u32) {
    let result = find_counter_example(size);

    if let Some(result) = result {
        println!("Error in the following example :");
        let mut x = Configuration::new(result, size);
        x.println();
        while ! x.has_converged() {
            x.update();
            x.println();
        }
    }
    else {
        println!("size {size} clean");
    }
}

/**
 * This function calls search_size for all sizes from 2 to 30, 30 included.
 * Expensive!
 */
fn search_all() {
    for size in 2..=30 {
        search_size(size);
    }
}

/**
 * This function generates a random initial configuration of a given size,
 * and prints all the steps of its execution on the terminal until it
 * converges. Useful for generating material to make figures in a scientific
 * article.
 */
fn show_random_execution(size : u32) {
    let mut x = Configuration::new(0, size);

    let mut rng = rand::thread_rng();

    x.value = rng.gen();
    x.value &= (1 << size) - 1;

    x.println();
    while ! x.has_converged() {
        x.update();
        x.println();
    }
}
