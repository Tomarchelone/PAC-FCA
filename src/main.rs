use std::time::Instant;

/*
mod pac_fca;
use crate::pac_fca::*;

mod div_oracle;
use crate::div_oracle::DivOracle;
*/


mod vec_pac_fca;
use crate::vec_pac_fca::*;

mod vec_div_oracle;
use crate::vec_div_oracle::DivOracle;


// Векторизовать операции с множествами?
// Добавить размер множества в ImplicationSet

fn main() {
    let M = 20;
    let eps = 0.01;
    let delta = 0.01;
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
    //println!("{}", {let mut l: u128 = rand::random(); l >> 127});
}
