use std::collections::HashMap;


#[derive(Debug, Clone)]
pub enum StringFieldValue {
    Exact(String),
    /// Possible values mapped to their probability. If the sum of the probabilities is not 1, the remaining probability is assigned to the "other" value.
    Distribution(HashMap<String, f64>),
    Error
}

impl StringFieldValue {
    pub fn merge(v: Vec<Self>) -> Self {
        //panic!("Not implemented");
        if v.len()==0 {
            return StringFieldValue::Error;
        }
        if v.len()==1{
            return v[0].clone();
        }
        // propagate errors
        if v.iter().any(|val| match val {
            StringFieldValue::Error => true,
            _ => false,
        }) {
            return StringFieldValue::Error;
        }
        let mut exacts= v.iter().filter_map(|val| match val {
            StringFieldValue::Exact(s) => Some(s),
            _ => None
        });
        if let Some(exact) = exacts.next() {
            if exacts.clone().all(|s| s==exact) {
                return StringFieldValue::Exact(exact.clone());
            }else {
                return StringFieldValue::Error;
            }
        }
        panic!("Not implemented");
        //now only distributions are left
        let mut result = HashMap::new();
        for val in v {
            match val {
                StringFieldValue::Exact(s) => {
                    let count = result.entry(s.clone()).or_insert(0.0);
                    *count += 1.0;
                }
                _ => {}
            }
        }
        let total: f64 = result.values().sum();
        for (_, count) in result.iter_mut() {
            *count /= total;
        }
        StringFieldValue::Distribution(result)
    }
}