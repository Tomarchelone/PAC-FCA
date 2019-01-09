extern crate rand;

use rand::random;
use std::time::Instant;

// Множество представляется как u128

pub trait Oracle {
    fn is_refuted(&self, imp: &Implication) -> Option<(u128, u128)>;
}

pub struct Context {
    pub M: usize,  // size of attribute set
    pub context: Vec<(u128, u128)>,  // pod-s
}

impl Context {
    fn allows(&self, p: u128) -> u128 {
        let mut ret = u128::max_value() >> (128 - self.M);
        for &(a, s) in &self.context {
            if is_subset(p, a) {
                for i in 0..self.M {
                    if contains(s, i) {
                        ret = remove(ret, i);
                    }
                }
            }
        }
        ret
    }
}

#[derive(Clone)]
pub struct Implication {
    pub from: u128,
    pub to: u128,
}

pub struct ImplicationSet {
    M: usize,
    pub set: Vec<Implication>,
}

impl ImplicationSet {
    // Самый примитивный алгоритм
    fn close(&self, mut l: u128) -> u128 {
        let mut used = vec![false; self.set.len()];  // if implication is used it cannot be used later
        let mut closed = false;
        while !closed {
            closed = true;
            for i in 0..self.set.len() {
                if used[i] {
                    continue;
                }
                if is_subset(self.set[i].from, l) {
                    let l_new = union(l, self.set[i].to);
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
        let M = self.M;
        let mut buf = String::new();
        for imp in &self.set {
            let mut assumption = "{".to_string();
            let mut new = true;
            for i in 0..M {
                if contains(imp.from, i) {
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
                if contains(imp.to, i) {
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

pub fn contains(set: u128, elem: usize) -> bool {
    (set & (1_u128 << elem)) != 0_u128
}

pub fn add(set: u128, elem: usize) -> u128 {
    set | (1_u128 << elem)
}

pub fn remove(set: u128, elem: usize) -> u128 {
    set & !(1_u128 << elem)
}

fn union(l: u128, e: u128) -> u128 {
    l | e
}

fn is_subset(sub: u128, set: u128) -> bool {
    sub & (!set) == 0_u128
}

fn random_subset(M: usize) -> u128 {
    let mut l: u128 = rand::random();
    l >> (128 - M)
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
              l = L.close(l);  // L-closure of l
              let r = K.allows(l);  // largest subset not refuted by K
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
        Some(some) => return some,
        None => return None,
    }
}

pub fn pac_attribute_exploration(
    M: usize
    , K_0: Context
    , eps: f64
    , delta: f64
    , oracle: &Oracle
)
    -> (ImplicationSet, Context)
{
    let mut K = K_0; // initial context
    let mut L = ImplicationSet {M, set: vec![]};
    let mut j = 1;
    loop {
        match probably_explored(M, &K, &L, eps, delta, j) {
            Some(mut imp) => {
                for i in 0..M {  // transform it to (l -> r \ l)
                    if contains(imp.from, i) {
                        imp.to = remove(imp.to, i);
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
