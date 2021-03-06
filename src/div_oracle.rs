// An implementation where n-th attribute means divisibility by n+2

extern crate num_bigint;
extern crate num_traits;

use num_bigint::BigUint;
use num_bigint::ToBigUint;
use num_traits::{Zero, One};

use crate::pac_fca::*;

fn gcd(a: &BigUint, b: &BigUint) -> BigUint {
    let (mut a, mut b) = (a.clone(), b.clone());
    while b != Zero::zero() {
        let c = a;
        a = b.clone();
        b = c % b;
    }
    a
}

fn lcm(a: BigUint, b: BigUint) -> BigUint {
    (&a * &b) / gcd(&a, &b)
}

pub struct DivOracle { // Oracle for numbers and divisibility
    pub M: usize,
}

impl Oracle for DivOracle {
    // НОК левой части должно делиться на всё, что в правой части
    fn is_refuted(&self, imp: &Implication) -> Option<(Vec<u128>, Vec<u128>)> {
        let M = self.M;
        let mut l = One::one();
        for i in 0..M {
            if contains(&imp.from, i) {
                l = lcm(l, (i + 2).to_biguint().unwrap())
            }
        }

        let mut true_divisors = empty_set(M);
        for i in 0..M {
            if &l % (i + 2).to_biguint().unwrap() == Zero::zero() {
                add(&mut true_divisors, i);
            }
        }

        for i in 0..M {
            if contains(&imp.to, i) && !contains(&true_divisors, i) {  // implication is wrong
                let not_divisors = not(&true_divisors, M);
                return Some((true_divisors, not_divisors));
            }
        }
        None
    }

    fn is_member(&self, example: &Vec<u128>) -> bool {
        true
    }
}
