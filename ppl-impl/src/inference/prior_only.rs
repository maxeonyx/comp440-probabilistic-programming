use crate::{
    distributions::Distribution,
    types::{RuntimeError, Value},
    DataFile,
};

use super::{flatten_to_numeric_vec_only, InferenceAlg};

pub struct PriorOnly {
    results: Vec<Value>,
}

impl PriorOnly {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }
}

impl InferenceAlg for PriorOnly {
    fn sample(
        &mut self,
        dist: &dyn Distribution,
        _sample_number: Option<usize>,
    ) -> Result<Value, RuntimeError> {
        dist.sample()
    }

    fn observe(
        &mut self,
        dist: &dyn Distribution,
        _val: Value,
        _observe_number: Option<usize>,
    ) -> Result<Value, RuntimeError> {
        dist.sample()
    }

    fn finish_one_evaluation(&mut self, result: Value) {
        self.results.push(result);
    }

    fn finalize_and_make_dataset(self) -> Result<DataFile, RuntimeError> {
        let vals = flatten_to_numeric_vec_only(self.results)?;

        Ok(DataFile {
            has_weights: false,
            data: vals,
        })
    }
}
