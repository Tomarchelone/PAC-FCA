extern crate rand;

use rand::random;
use std::time::Instant;

pub trait Oracle {
    fn is_refuted(&self, imp: &Implication) -> Option<(Vec<u128>, Vec<u128>)>;

    fn is_member(&self, example: &Vec<u128>) -> bool;
}

pub struct Context {
    pub M: usize,  // size of attribute set
    pub context: Vec<(Vec<u128>, Vec<u128>)>,  // pod-s
}

impl Context {
    fn allows(&self, p: &Vec<u128>) -> Vec<u128> {
        let mut ret = full_set(self.M);
        for (a, s) in &self.context {
            if is_subset(p, a) {
                for i in 0..self.M {
                    if contains(s, i) {
                        remove(&mut ret, i);
                    }
                }
            }
        }
        ret
    }
}


#[derive(Clone)]
pub struct Implication {
    pub from: Vec<u128>,
    pub to: Vec<u128>,
}

pub struct ImplicationSet {
    pub M: usize,
    pub set: Vec<Implication>,
}

impl ImplicationSet {
    // Самый примитивный алгоритм
    fn close(&self, l: &Vec<u128>) -> Vec<u128> {
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
                    let mut l_new = l.clone();
                    union(&mut l_new, &self.set[i].to);
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
                if contains(&imp.from, i) {
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
                if contains(&imp.to, i) {
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

#[inline]
pub fn full_set(M: usize) -> Vec<u128> {
    let chunks = M / 128;
    let rem = M % 128;
    let mut ret = vec![u128::max_value(); chunks];
    if rem != 0 {
        ret.push(u128::max_value() >> (128 - rem));
    }
    ret
}

#[inline]
pub fn empty_set(M: usize) -> Vec<u128> {
    let chunks = M / 128;
    let rem = M % 128;
    if rem == 0 {
        vec![0_u128; chunks]
    } else {
        vec![0_u128; chunks + 1]
    }
}

#[inline]
pub fn not(set: &Vec<u128>, M: usize) -> Vec<u128> {
    let chunks = M / 128;
    let rem = M % 128;
    let mut ret = vec![0_u128; chunks];
    for i in 0..chunks {
        ret[i] = !set[i];
    }
    if rem != 0 {
        ret.push((!set[chunks] << (128 - rem)) >> (128 - rem));
    }
    ret
}

#[inline]
pub fn contains(set: &Vec<u128>, elem: usize) -> bool {
    let chunk = elem / 128;
    let bit = elem % 128;
    (set[chunk] & (1_u128 << bit)) != 0_u128
}

#[inline]
pub fn add(set: &mut Vec<u128>, elem: usize) {
    let chunk = elem / 128;
    let bit = elem % 128;
    set[chunk] = set[chunk] | (1_u128 << bit);
}

#[inline]
pub fn remove(set: &mut Vec<u128>, elem: usize) {
    let chunk = elem / 128;
    let bit = elem % 128;
    set[chunk] = set[chunk] & !(1_u128 << bit);
}

#[inline]
pub fn union(l: &mut Vec<u128>, e: &Vec<u128>) {
    for i in 0..l.len() {
        l[i] = l[i] | e[i];
    }
}

#[inline]
pub fn is_subset(sub: &Vec<u128>, set: &Vec<u128>) -> bool {
    for i in 0..sub.len() {
        if !(sub[i] & (!set[i]) == 0_u128) {
            return false
        }
    }
    true
}

#[inline]
fn random_subset(M: usize) -> Vec<u128> {
    let chunks = M / 128;
    let rem = M % 128;
    let mut ret = vec![0_u128; chunks];
    for i in 0..chunks {
        ret[i] = rand::random::<u128>();
    }
    let tail: u128 = rand::random();
    if rem != 0 {
        ret.push(tail >> (128 - rem));
    }
    ret
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
    'main: loop {
        match probably_explored(M, &K, &L, eps, delta, j) {
            Some(mut imp) => {
                for i in 0..M {  // transform it to (l -> r \ l)
                    if contains(&imp.from, i) {
                        remove(&mut imp.to, i);
                    }
                }
                println!("Member with L size {}", L.set.len());
                match oracle.is_refuted(&imp) {
                    Some(pod) => {
                        K.context.push(pod);
                    }
                    None => {
                        let mut found = false;
                        let mut k = 0_usize;
                        while k < L.set.len() {
                            if is_subset(&imp.from, &L.set[k].from) && is_subset(&L.set[k].to, &imp.to) {
                                if !found {
                                    L.set[k] = imp.clone();
                                    found = true;
                                } else {
                                    L.set.swap_remove(k);
                                }
                            }
                            k += 1;
                        }
                        if !found {
                            L.set.push(imp);
                        }


                        for i in 0..K.context.len() {
                            K.context[i].0 = L.close(&K.context[i].0);
                        }

                    }
                }
            },
            None => return (L, K),
        }
        j += 1;
    }
}
