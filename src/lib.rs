#![feature(io_error_more)]
#![allow(unused)]

mod storage;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn calculate_hash(t: &str) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        for i in 0..10 {
            let result = calculate_hash("calculate");
            println!("{}",format!("{result:X}"));
            assert_eq!(result, 797949694947597695);
            let result = calculate_hash("сalculate");
            println!("{}",format!("{result:#X}"));
        }


    }
}
