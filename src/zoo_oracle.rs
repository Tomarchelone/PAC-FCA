use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;

use crate::pac_fca::*;


pub struct ZooOracle {
    pub M: usize,
    animals: Vec<Vec<u128>>,
}

impl ZooOracle {
    pub fn from_data(mut data: File) -> ZooOracle {
        let mut contents = String::new();
        data.read_to_string(&mut contents).unwrap();

        let mut leg_idx = HashMap::new();
        let leg_counts = ["0", "2", "4", "5", "6", "8"];
        for i in 0..leg_counts.len() {
            leg_idx.insert(leg_counts[i], i);
        }

        let mut M = 0;
        let mut animals: Vec<_> = contents
                                .split_whitespace()
                                .map(|x| {
                                    let raw_attributes = x.split(",").collect::<Vec<_>>();

                                    let mut attributes = raw_attributes[1..13].to_vec();
                                    let mut tail = raw_attributes[14..17].to_vec();
                                    attributes.append(&mut tail);
                                    let mut attributes: Vec<bool> = attributes.iter().cloned().map(|x| {
                                        x == "1"
                                    }).collect();

                                    let legs = raw_attributes[13];
                                    let mut leg_vec = vec![false; 6];
                                    leg_vec[leg_idx[legs]] = true;

                                    attributes.append(&mut leg_vec);
                                    M = attributes.len();

                                    from_bool(attributes)
                                })
                                .collect();
        ZooOracle { M, animals }
    }
}

impl Oracle for ZooOracle {
    fn is_refuted(&self, imp: &Implication) -> Option<(Vec<u128>, Vec<u128>)> {
        let M = self.M;
        for animal in &self.animals {
            if is_subset(&imp.from, animal) && !is_subset(&imp.to, animal) {
                return Some((animal.clone(), not(&animal, M)));
            }
        }
        None
    }

    fn is_member(&self, example: &Vec<u128>) -> bool { // 1 means examle has attribute, 0 means nothing
        for animal in &self.animals {
            if is_subset(example, animal) {
                return true;
            }
        }
        false
    }
}


fn from_bool(v: Vec<bool>) -> Vec<u128> {
    let M = v.len();
    let chunks = M / 128;
    let rem = M % 128;
    let mut ret = vec![0_u128; chunks];

    for i in 0..chunks {
        for j in 0..128 {
            if v[i * 128 + j] {
                ret[i] = ret[i] | (1_u128 << j);
            }
        }
    }
    if rem != 0 {
        let mut tail = 0_u128;
        for j in 0..rem {
            if v[chunks * 128 + j] {
                tail = tail | (1_u128 << j);
            }
        }
        ret.push(tail);
    }

    ret
}

pub fn zoo_imp(impset: &ImplicationSet) -> String {
    let attr_text = ["hair", "feathers", "eggs", "milk", "airborne", "aquatic"
                     , "predator", "toothed", "backbone", "breathes", "venomous"
                     , "fins", "tail", "domestic", "catsize"
                     , "0 legs", "2 legs", "4 legs", "5 legs", "6 legs", "8 legs"];
    let M = impset.M;
    let mut ret = String::new();

    for imp in &impset.set {
        let mut assumption = "{".to_string();
        let mut new = true;
        for i in 0..M {
            if contains(&imp.from, i) {
                if new {
                    assumption.push_str(&format!("{}", attr_text[i]));
                    new = false;
                } else {
                    assumption.push_str(&format!(", {}", attr_text[i]));
                }
            }
        }
        assumption.push('}');

        let mut conclusion = "{".to_string();
        let mut new = true;
        for i in 0..M {
            if contains(&imp.to, i) {
                if new {
                    conclusion.push_str(&format!("{}", attr_text[i]));
                    new = false;
                } else {
                    conclusion.push_str(&format!(", {}", attr_text[i]));
                }
            }
        }
        conclusion.push('}');

        ret.push_str(&format!("{} -> {}\n", assumption, conclusion));
    }

    ret
}
