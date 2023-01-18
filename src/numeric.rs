use std::f64::NAN;

use peroxide::{fuga::Integral::*, numerical::integral::*};
use serde::{Serialize, Deserialize};
#[derive(Debug, Clone,Serialize,Deserialize)]
pub enum NumericFieldValue {
    Normal {
        sigma: f64,
        mean: f64,
    },
    Exact(f64),
    Uniform {
        min: f64,
        max: f64,
    },
    Combination {
        components: Vec<NumericFieldValue>,
        scaling_factor: f64,
    },
    Error,
}

async_graphql::scalar!(NumericFieldValue);
#[derive(Debug, Clone)]
pub struct DistributionPlot {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
}

impl NumericFieldValue {
    pub fn get_value(&self, x: f64) -> f64 {
        match self {
            NumericFieldValue::Normal { sigma, mean } => {
                (1.0 / (sigma * 2.0 * std::f64::consts::PI).sqrt())
                    * (-0.5 * ((x - mean) / sigma).powi(2)).exp()
            }
            NumericFieldValue::Exact(v) => {
                if *v == x {
                    1.0
                } else {
                    0.0
                }
            }
            NumericFieldValue::Uniform { min, max } => {
                if x >= *min && x <= *max {
                    1.0 / (max - min)
                } else {
                    0.0
                }
            }
            NumericFieldValue::Combination {
                components,
                scaling_factor,
            } => {
                let result: f64 = components.into_iter().map(|val| val.get_value(x)).product();
                result * scaling_factor
            }
            NumericFieldValue::Error => NAN,
        }
    }

    pub fn mean(&self) -> f64 {
        match self {
            NumericFieldValue::Normal { sigma: _, mean } => *mean,
            NumericFieldValue::Exact(v) => *v,
            NumericFieldValue::Uniform { min, max } => (min + max) / 2.0,
            NumericFieldValue::Combination {
                components,
                scaling_factor: _,
            } => components.iter().map(|val| val.mean()).sum::<f64>() / (components.len() as f64),
            NumericFieldValue::Error => NAN,
        }
    }

    pub fn sigma(&self) -> f64 {
        match self {
            NumericFieldValue::Normal { sigma, mean: _ } => *sigma,
            NumericFieldValue::Exact(_) => 0.0,
            NumericFieldValue::Uniform { min, max } => (max - min) / 12.0,
            NumericFieldValue::Combination {
                components,
                scaling_factor: _,
            } => components.iter().map(|val| val.sigma()).sum::<f64>() / (components.len() as f64),
            NumericFieldValue::Error => NAN,
        }
    }

    pub fn merge(v: Vec<Self>) -> Self {
        // propagate errors
        if v.iter().any(|val| match val {
            NumericFieldValue::Error => true,
            _ => false,
        }) {
            return NumericFieldValue::Error;
        }
        let mut exacts = v.iter().filter_map(|val| match val {
            NumericFieldValue::Exact(v) => Some(v),
            _ => None,
        });
        if let Some(e) = exacts.next() {
            if exacts.all(|v| v == e) {
                return NumericFieldValue::Exact(*e);
            }
            return NumericFieldValue::Error;
        }
        let mean = v.iter().map(|val| val.mean()).sum::<f64>() / (v.len() as f64);
        let sigma = v.iter().map(|val| val.sigma()).sum::<f64>() / (v.len() as f64);
        let range = (mean - 3.0 * sigma, mean + 3.0 * sigma);
        let area = integrate(
            |x1| v.iter().map(|val| val.get_value(x1)).product(),
            range,
            G20K41(1.0e-3),
        );
        return NumericFieldValue::Combination {
            components: v,
            scaling_factor: 1.0 / area,
        };
    }

    pub fn get_distribution(&self, steps: usize) -> DistributionPlot {
        let x: Vec<f64> = (0..steps)
            .map(|i| (i as f64 / (steps as f64) - 0.5) * 2.0 * 2.0 * self.sigma() + self.mean())
            .collect();
        let y = x.clone().into_iter().map(|x| self.get_value(x)).collect();

        DistributionPlot { x, y }
    }
}
