use std::time::Instant;
use std::fs::File;

mod pac_fca;
use crate::pac_fca::*;

mod div_oracle;
use crate::div_oracle::DivOracle;

mod zoo_oracle;
use crate::zoo_oracle::*;

// Добавить пустой дизъюнкт?

fn main() {
    let M = 18;
    let eps = 0.01;
    let delta = 0.01;
    let oracle = DivOracle {M};
    // let oracle = ZooOracle::from_data(File::open("zoo.data").unwrap());
    // let M = oracle.M;

    let mut stamp = Instant::now();
    let (imp_set, context) = pac_attribute_exploration(
                                M
                                , Context {M, context: vec![]}
                                , eps
                                , delta
                                , &oracle
                            );
    let elapsed = stamp.elapsed();

    println!("{}", imp_set);
    println!("[Elapsed: {:?}]", stamp.elapsed());
    println!("[Size of implication set: {}]", imp_set.set.len());
}
