use std::f64::NAN;

use peroxide::{fuga::Integral::*, numerical::integral::*};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize,JsonSchema)]
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
        mean: f64,
        sigma: f64,
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
                (1.0 / sigma / (2.0 * std::f64::consts::PI).sqrt())
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
                ..
            } => {
                let result: f64 = components.into_iter().map(|val| val.get_value(x)).product();
                result * scaling_factor
            }
            NumericFieldValue::Error => f64::NAN,
        }
    }

    pub fn mean(&self) -> f64 {
        match self {
            NumericFieldValue::Normal { sigma: _, mean } => *mean,
            NumericFieldValue::Exact(v) => *v,
            NumericFieldValue::Uniform { min, max } => (min + max) / 2.0,
            NumericFieldValue::Combination { mean, .. } => *mean,
            NumericFieldValue::Error => NAN,
        }
    }

    pub fn sigma(&self) -> f64 {
        match self {
            NumericFieldValue::Normal { sigma, mean: _ } => *sigma,
            NumericFieldValue::Exact(_) => 0.0,
            NumericFieldValue::Uniform { min, max } => (max - min) / 12.0_f64.sqrt(),
            NumericFieldValue::Combination { sigma, .. } => *sigma,
            NumericFieldValue::Error => NAN,
        }
    }
    /// takes a callback that maps x and the probability density at x to the value to be integrated
    pub fn integrate<F>(&self, f: F) -> f64
    where
        F: Fn(f64, f64) -> f64,
    {
        let range = (
            self.mean() - 3.0 * self.sigma(),
            self.mean() + 3.0 * self.sigma(),
        );
        integrate(|x1| f(x1, self.get_value(x1)), range, G20K41(1.0e-3))
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
        let mean_approx = v.iter().map(|val| val.mean()).sum::<f64>() / (v.len() as f64);
        let sigma_approx = v.iter().map(|val| val.sigma()).sum::<f64>() / (v.len() as f64);
        let range = (
            mean_approx - 3.0 * sigma_approx,
            mean_approx + 3.0 * sigma_approx,
        );
        let area = integrate(
            |x1| v.iter().map(|val| val.get_value(x1)).product(),
            range,
            G20K41(1.0e-3),
        );
        let mean = integrate(
            |x1| x1 * v.iter().map(|val| val.get_value(x1)).product::<f64>() / area,
            range,
            G20K41(1.0e-3),
        );
        let variance = integrate(
            |x1| {
                (x1 - mean) * (x1 - mean) * v.iter().map(|val| val.get_value(x1)).product::<f64>()
                    / area
            },
            range,
            G20K41(1.0e-3),
        );
        return NumericFieldValue::Combination {
            components: v,
            scaling_factor: 1.0 / area,
            mean: mean,
            sigma: variance.sqrt(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform() {
        let uf = NumericFieldValue::Uniform {
            min: -0.1,
            max: 0.5,
        };
        assert!(
            (uf.integrate(|_x, f| f) - 1.0).abs() < 0.01,
            "is no probability distribution"
        );
        assert!(
            (uf.integrate(|x, f| f * x) - 0.2).abs() < 0.01,
            "has wrong mean"
        );
        assert!(
            (uf.integrate(|x, f| f * (x - 0.2).powi(2)) - 0.6 * 0.6 / 12.0).abs() < 0.01,
            "has wrong variance"
        );
    }
    #[test]
    fn normal() {
        let uf = NumericFieldValue::Normal {
            sigma: 0.4,
            mean: 0.2,
        };
        assert!(
            (uf.integrate(|_x, f| f) - 1.0).abs() < 0.01,
            "is no probability distribution"
        );
        assert!(
            (uf.integrate(|x, f| f * x) - 0.2).abs() < 0.01,
            "has wrong mean"
        );
        assert!(
            (uf.integrate(|x, f| f * (x - 0.2).powi(2)) - 0.4 * 0.4).abs() < 0.01,
            "has wrong variance"
        );
    }
    #[test]
    fn combination() {
        let uf = NumericFieldValue::merge(vec![
            NumericFieldValue::Uniform {
                min: -0.1,
                max: 0.5,
            },
            NumericFieldValue::Uniform { min: 0.2, max: 0.4 },
        ]);
        assert!(
            (uf.integrate(|_x, f| f) - 1.0).abs() < 0.01,
            "is no probability distribution"
        );
        assert!(
            (uf.integrate(|x, f| f * x) - 0.3).abs() < 0.01,
            "has wrong mean"
        );
        assert!(
            (uf.integrate(|x, f| f * (x - 0.2).powi(2)) - 0.2 * 0.2 / 12.0).abs() < 0.01,
            "has wrong variance"
        );
    }
}
