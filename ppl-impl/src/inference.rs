use std::error::Error;

use crate::{DataFile, IntOrFloat, ProgramResult, types::{Distribution, RuntimeError, Value}};

fn flatten_to_numeric_vec_only(vals: Vec<Value>) -> Result<Vec<ProgramResult>, RuntimeError> {
    vals.into_iter()
        .map(|v| match v {
            Value::Integer(i) => Ok(ProgramResult::One(IntOrFloat::Int(i))),
            Value::Float(f) => Ok(ProgramResult::One(IntOrFloat::Float(f))),
            Value::Vector(v) => Ok(ProgramResult::Many(flatten_to_numeric_vec_only(v)?)),
            _ => err!("Program should only return numbers or vecs of numbers."),
        })
        .collect::<Result<Vec<ProgramResult>, RuntimeError>>()
}

pub trait InferenceAlg {
    fn sample(&mut self, dist: &dyn Distribution) -> Result<Value, RuntimeError>;
    fn observe(&mut self, dist: &dyn Distribution, val: Value) -> Result<Value, RuntimeError>;

    fn finish_one_evaluation(&mut self, result: Value);
    fn finalize_and_write(&self) -> Result<DataFile, RuntimeError>;
}

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
    fn sample(&mut self, dist: &dyn Distribution) -> Result<Value, RuntimeError> {
        dist.sample()
    }

    fn observe(&mut self, dist: &dyn Distribution, val: Value) -> Result<Value, RuntimeError> {
        dist.sample()
    }

    fn finish_one_evaluation(&mut self, result: Value) {
        self.results.push(result);
    }

    fn finalize_and_write(&self) -> Result<DataFile, RuntimeError> {
        let vals = flatten_to_numeric_vec_only(self.results)?;
 
        Ok(DataFile {
            has_weights: false,
            data: vals,
        })
    }
}

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
    fn sample(&mut self, dist: &dyn Distribution) -> Result<Value, RuntimeError> {
        dist.sample()
    }

    fn observe(&mut self, dist: &dyn Distribution, val: Value) -> Result<Value, RuntimeError> {

        self.log_w += dist.pdf(val.clone())?.ln();

        Ok(val)
    }
    fn finish_one_evaluation(&mut self, result: Value) {
        let log_w = self.log_w;
        self.log_w = 0f64;
        self.results.push(result);
        self.weights.push(log_w);
    }

    fn finalize_and_write(&self)  ->  Result<DataFile, RuntimeError> {
        let vals = flatten_to_numeric_vec_only(self.results.to_vec())?;
        Ok(DataFile {
            has_weights: true,
            data: vals.into_iter().zip(self.weights.iter()).map(|(val, weight)| ProgramResult::Many(vec![val, ProgramResult::One(IntOrFloat::Float(*weight))])).collect()
        })
        
    }
}
