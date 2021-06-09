use std::collections::HashMap;

use crate::{DataFile, distributions::Distribution, types::{RuntimeError, Value}};

use super::InferenceAlg;


enum ProgramVar {
    Sample,
    Observe,
}

type Location = (usize, usize);

struct SingleSiteMetropolis {
    
    // counter for the 
    var_counter: usize,

    // Remember the program trace
    last_program_trace: HashMap<(usize, usize), Option<Value>>,
    last_program_log_weight: f64,

    proposal_program_trace: HashMap<(usize, usize), Option<Value>>,
    proposal_program_log_weight: f64,

    // all_program traces
    samples: Vec<Value>,
}


impl InferenceAlg for SingleSiteMetropolis {
    
    fn sample(&mut self, dist: &dyn Distribution, sample_number: Option<usize>) -> Result<Value, RuntimeError> {
        let sample_number = sample_number.expect("Shouldn't happen: SingleSiteMetropolis didn't recieve a sample_number.");
        let key = (sample_number, self.var_counter);
        self.var_counter += 1;
        let (val, weight) = match self.last_program_trace.get(&key) {
            // reuse the value from the previous run.
            Some(val) => val.clone().expect("Shouldn't happen: Previous trace of a sample was an observe."),

            // the variable wasn't in the previous run.
            None => {
                let val = dist.sample()?;
                // todo add log weight
                self.last_program_trace.insert(key, Some(val.clone()));
                (val, log_weight)
            },
        };
        let log_weight = dist.log_pdf(&val)?;

        self.proposal_program_log_weight += weight;

        Ok(val)
    }

    fn observe(&mut self, dist: &dyn Distribution, val: Value, observe_number: Option<usize>) -> Result<Value, RuntimeError> {
        let observe_number = observe_number.expect("Shouldn't happen: SingleSiteMetropolis didn't recieve a observe_number.");
        let key = (observe_number, self.var_counter);
        self.var_counter += 1;
        match self.last_program_trace.get(&key) {
            // reuse the value from the previous run.
            Some((val, prob)) => {

            },

            // the variable wasn't in the previous run.
            None => {
                // todo add log weight
                let log_weight = dist.log_pdf(&val)?;
                self.last_program_trace.insert(key, (None, log_weight));
            },
        };
        Ok(val)

    }

    fn finish_one_evaluation(&mut self, val: Value) {
        self.var_counter = 0;
        self.samples.push(val)
    }

    fn finalize_and_make_dataset(self) -> Result<DataFile, RuntimeError> {
        todo!()
    }
}
