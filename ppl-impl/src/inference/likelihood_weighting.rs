use crate::{DataFile, IntOrFloat, ProgramResult, distributions::Distribution, types::{RuntimeError, Value}};

use super::{flatten_to_numeric_vec_only, InferenceAlg};

pub struct LikelihoodWeighting {
    pub log_w: f64,
    pub results: Vec<Value>,
    pub weights: Vec<f64>,
}

impl LikelihoodWeighting {
    pub fn new() -> Self {
        Self {
            log_w: 0f64,
            results: Vec::new(),
            weights: Vec::new(),
        }
    }
}

impl InferenceAlg for LikelihoodWeighting {
    fn sample(&mut self, dist: &dyn Distribution, _sample_number: Option<usize>) -> Result<Value, RuntimeError> {
        dist.sample()
    }

    fn observe(&mut self, dist: &dyn Distribution, val: Value, _observe_number: Option<usize>) -> Result<Value, RuntimeError> {
        self.log_w += dist.log_pdf(&val)?;

        Ok(val)
    }
    fn finish_one_evaluation(&mut self, result: Value) {
        let log_w = self.log_w;
        self.log_w = 0f64;
        self.results.push(result);
        self.weights.push(log_w);
    }

    fn finalize_and_make_dataset(self) -> Result<DataFile, RuntimeError> {
        let vals = flatten_to_numeric_vec_only(self.results.to_vec())?;
        Ok(DataFile {
            has_weights: true,
            data: vals
                .into_iter()
                .zip(self.weights.iter())
                .map(|(val, weight)| {
                    ProgramResult::Many(vec![val, ProgramResult::One(IntOrFloat::Float(*weight))])
                })
                .collect(),
        })
    }
}
