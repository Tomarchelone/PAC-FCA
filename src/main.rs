extern crate rand;

use rand::random;
use std::time::Instant;

// All pods are fods here

trait Oracle {
    fn is_refuted(&self, imp: &Implication) -> Option<(Vec<bool>, Vec<bool>)>;
}

struct Context {
    pub M: usize,  // size of attribute set
    pub context: Vec<(Vec<bool>, Vec<bool>)>,  // pod-s
}

impl Context {
    fn allows(&self, p: &Vec<bool>) -> Vec<bool> {
        let mut ret = vec![true; self.M];
        for (a, s) in &self.context {
            if is_subset(p, a) {
                for i in 0..self.M {
                    if s[i] {
                        ret[i] = false;
                    }
                }
            }
        }
        ret
    }
}

#[derive(Clone)]
struct Implication {
    pub from: Vec<bool>,
    pub to: Vec<bool>,
}

struct ImplicationSet {
    pub set: Vec<Implication>,
}

impl ImplicationSet {
    // Самый примитивный алгоритм
    fn close(&self, l: &Vec<bool>) -> Vec<bool> {
        let mut l = l.clone();
        let mut used = vec![false; self.set.len()];  // if implication is used it cannot be used later
        let mut closed = false;
        while !closed {
            closed = true;
            for i in 0..self.set.len() {
                if used[i] {
                    continue;
                }
                if is_subset(&self.set[i].from, &l) {
                    let l_new = extend(&l, &self.set[i].to);
                    if l_new != l {
                        closed = false;
                        l = l_new;
                        used[i] = true;
                    }
                }
            }
        }
        l
    }
}

use std::fmt;
impl fmt::Display for ImplicationSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let M = self.set[0].from.len();
        let mut buf = String::new();
        for imp in &self.set {
            let mut assumption = "{".to_string();
            let mut new = true;
            for i in 0..M {
                if imp.from[i] {
                    if new {
                        assumption.push_str(&format!("{}", i + 2));
                        new = false;
                    } else {
                        assumption.push_str(&format!(", {}", i + 2));
                    }
                }
            }
            assumption.push('}');

            let mut conclusion = "{".to_string();
            let mut new = true;
            for i in 0..M {
                if imp.to[i] {
                    if new {
                        conclusion.push_str(&format!("{}", i + 2));
                        new = false;
                    } else {
                        conclusion.push_str(&format!(", {}", i + 2));
                    }
                }
            }
            conclusion.push('}');

            buf.push_str(&format!("{} -> {}\n", assumption, conclusion));
        }
        write!(f, "{}", buf)
    }
}

fn extend(l: &Vec<bool>, e: &Vec<bool>) -> Vec<bool> {
    let mut l_new = l.clone();
    for i in 0..l.len() {
        if e[i] {
            l_new[i] = true;
        }
    }
    l_new
}

fn is_subset(sub: &Vec<bool>, set: &Vec<bool>) -> bool {
    for i in 0..sub.len() {
        if sub[i] && !set[i] {
            return false;
        }
    }
    true
}


fn random_subset(M: usize) -> Vec<bool> {
    let mut l = vec![false; M];
    for i in 0..M {
        l[i] = random();
    }
    l
}


extern crate rayon;
use rayon::prelude::*;

fn probably_explored(
    M: usize  // Attribute set 0..M
    , K: &Context  // Context
    , L: &ImplicationSet  // Implication set
    , eps: f64
    , delta: f64
    , j: u64
)
    -> Option<Implication>
{
    match (0..((1.0/eps).ceil() as u64 * (j + (1.0/delta).ln().ceil() as u64)))
          .into_par_iter() // parallel search
          .map(|_| {
              let mut l = random_subset(M);
              l = L.close(&l);  // L-closure of l
              let r = K.allows(&l);  // largest subset not refuted by K
              if l != r {
                  return Some(Implication {from: l, to: r})  // "l -> r" is undecided
              }
              return None;
          })
          .find_any(
              |x| match x {
                  Some(_) => true,
                  _ => false,
              }
          )
    {
        Some(imp) => return imp,
        None => return None,
    }
}

fn pac_attribute_exploration(
    M: usize
    , K_0: Context
    , eps: f64
    , delta: f64
    , oracle: &Oracle
)
    -> (ImplicationSet, Context)
{
    let mut K = K_0; // initial context
    let mut L = ImplicationSet {set: vec![]};
    let mut j = 1;
    loop {
        match probably_explored(M, &K, &L, eps, delta, j) {
            Some(mut imp) => {
                for i in 0..M {  // transform it to (l -> r \ l)
                    if imp.from[i] {
                        imp.to[i] = false;
                    }
                }
                match oracle.is_refuted(&imp) {
                    Some(pod) => {
                        K.context.push(pod);
                    }
                    None => {
                        L.set.push(imp);
                        /*
                        for i in 0..K.context.len() {
                            K.context[i].0 = L.close(&K.context[i].0);
                        }
                        */
                    }
                }
            },
            None => return (L, K),
        }
        println!("Iteration {}", j);
        j += 1;
    }
}

// -------------------------------------------------------------------------
// Below is an implementation where n-th attribute means divisibility by n+2

extern crate num_bigint;
extern crate num_traits;

use num_bigint::BigUint;
use num_bigint::ToBigUint;
use num_traits::{Zero, One};

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

struct DivOracle {}  // Oracle for numbers and divisibility

impl Oracle for DivOracle {
    // НОК левой части должно делиться на всё, что в правой части
    fn is_refuted(&self, imp: &Implication) -> Option<(Vec<bool>, Vec<bool>)> {
        let M = imp.from.len();
        let mut l = One::one();
        for i in 0..M {
            if imp.from[i] {
                l = lcm(l, (i + 2).to_biguint().unwrap())
            }
        }

        let mut true_divisors = vec![false; imp.to.len()];
        for i in 0..M {
            if &l % (i + 2).to_biguint().unwrap() == Zero::zero() {
                true_divisors[i] = true;
            }
        }

        for i in 0..M {
            if imp.to[i] && !true_divisors[i] {  // implication is wrong
                let mut not_divisors = vec![false; M];
                for i in 0..M {
                    not_divisors[i] = !true_divisors[i];
                }
                return Some((true_divisors, not_divisors));
            }
        }
        None
    }
}

fn main() {
    let oracle = DivOracle {};
    let M = 14;
    let eps = 0.1;
    let delta = 0.1;

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
