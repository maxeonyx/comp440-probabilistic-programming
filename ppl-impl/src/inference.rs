use crate::{
    distributions::Distribution,
    types::{RuntimeError, Value},
    DataFile, ResultValue, ProgramResult,
};

fn flatten_to_numeric_vec_only(vals: Vec<Value>) -> Result<Vec<ProgramResult>, RuntimeError> {
    vals.into_iter()
        .map(|v| match v {
            Value::Integer(x) => Ok(ProgramResult::One(ResultValue::Int(x))),
            Value::Boolean(x) => Ok(ProgramResult::One(ResultValue::Boolean(x))),
            Value::Float(x) => Ok(ProgramResult::One(ResultValue::Float(x))),
            Value::Vector(x) => Ok(ProgramResult::Many(flatten_to_numeric_vec_only(x)?)),
            _ => err!("Program should only return numbers or vecs of numbers."),
        })
        .collect::<Result<Vec<ProgramResult>, RuntimeError>>()
}

pub trait InferenceAlg {
    fn sample(
        &mut self,
        dist: &dyn Distribution,
        sample_number: Option<usize>,
    ) -> Result<Value, RuntimeError>;
    fn observe(
        &mut self,
        dist: &dyn Distribution,
        val: Value,
        observe_number: Option<usize>,
    ) -> Result<Value, RuntimeError>;

    fn finish_one_evaluation(&mut self, val: Value);
    fn finalize_and_make_dataset(self) -> Result<DataFile, RuntimeError>;
}

pub mod likelihood_weighting;
pub mod prior_only;
pub mod single_site_metropolis;
