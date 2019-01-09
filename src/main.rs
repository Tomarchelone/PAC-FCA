use std::time::Instant;

mod pac_fca;
use crate::pac_fca::*;

mod div_oracle;
use crate::div_oracle::DivOracle;

fn main() {
    let M = 14;
    let eps = 0.1;
    let delta = 0.1;
    let oracle = DivOracle {M};

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
