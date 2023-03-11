use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize,JsonSchema)]
pub enum StringFieldValue {
    Exact(String),
    /// Possible values mapped to their probability. If the sum of the probabilities is not 1, the remaining probability is assigned to the "other" value.
    Distribution(HashMap<String, f64>),
    Error,
}

impl StringFieldValue {
    pub fn merge(v: Vec<Self>) -> Self {
        //panic!("Not implemented");
        if v.len() == 0 {
            return StringFieldValue::Error;
        }
        if v.len() == 1 {
            return v[0].clone();
        }
        // propagate errors
        if v.iter().any(|val| match val {
            StringFieldValue::Error => true,
            _ => false,
        }) {
            return StringFieldValue::Error;
        }
        let mut exacts = v.iter().filter_map(|val| match val {
            StringFieldValue::Exact(s) => Some(s),
            _ => None,
        });
        if let Some(exact) = exacts.next() {
            if exacts.clone().all(|s| s == exact) {
                return StringFieldValue::Exact(exact.clone());
            } else {
                return StringFieldValue::Error;
            }
        }
        //now only distributions are left
        let mut result = HashMap::new();
        let distributions = v.iter().filter_map(|val| match val {
            StringFieldValue::Distribution(s) => Some(s),
            _ => None,
        });
        let first_distribution = match distributions.clone().next() {
            Some(d) => d,
            None => return StringFieldValue::Error,
        };
        let mut probability_sum = 0.0;
        for (key, _) in first_distribution {
            let probability = distributions
                .clone()
                .map(|d| d.get(key).unwrap_or(&0.0))
                .product();
            probability_sum += probability;
            if probability > 0.0 {
                result.insert(key.clone(), probability);
            }
        }
        if probability_sum==0.0{
            return StringFieldValue::Error;
        }
        for (_, value) in result.iter_mut() {
            *value /= probability_sum;
        }
        StringFieldValue::Distribution(result)
    }
}
