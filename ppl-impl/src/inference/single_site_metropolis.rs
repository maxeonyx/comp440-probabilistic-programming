use std::collections::HashMap;

use rand::{thread_rng, Rng};

use crate::{
    distributions::Distribution,
    types::{RuntimeError, Value},
    DataFile,
};

use super::{flatten_to_numeric_vec_only, InferenceAlg};

// refers to a unique instantiation of a `sample` or `observe` expression.
type VariableAddress = (usize, usize);
type Trace = HashMap<VariableAddress, (Value, f64)>;

pub struct SingleSiteMetropolis {
    skip: usize,
    count: usize,

    // Remember the program trace
    last: Option<RunMemory>,
    proposal: RunMemory,

    reached_proposal_site: bool,
    proposal_site: VariableAddress,
    var_counter: usize,

    // all_program traces
    samples: Vec<Value>,
}

struct RunMemory {
    trace: Trace,
    reused_log_weight: f64,
    observed_log_weight: f64,
}

impl SingleSiteMetropolis {
    pub fn new(skip: usize) -> Self {
        Self {
            skip,
            count: 0,
            last: None,
            proposal: RunMemory {
                trace: HashMap::new(),
                reused_log_weight: 0.,
                observed_log_weight: 0.,
            },
            reached_proposal_site: false,
            proposal_site: (0, 0),
            var_counter: 0,
            samples: Vec::new(),
        }
    }
}

impl InferenceAlg for SingleSiteMetropolis {
    fn sample(
        &mut self,
        dist: &dyn Distribution,
        sample_number: Option<usize>,
    ) -> Result<Value, RuntimeError> {
        let sample_number = sample_number
            .expect("Shouldn't happen: SingleSiteMetropolis didn't recieve a sample_number.");

        // construct a unique identifier for a random variable based on:
        // - The `sample` statement it represents
        // - The number of times any random variable has been sampled/observed in this program execution.
        // This is slightly sketchy, I'm pretty sure I can construct a program which causes nonsense
        let key = (sample_number, self.var_counter);
        self.var_counter += 1;

        let (val, log_weight) = if let Some(last) = &mut self.last {
            if key == self.proposal_site {
                // if this variable is the proposal site
                self.reached_proposal_site = true;
                let val = dist.sample()?;
                let log_weight = dist.log_pdf(&val)?;
                (val, log_weight)
            } else if let Some((val, log_weight)) = last.trace.get(&key) {
                // otherwise, if it is part of the previous run.

                // If we have reached the proposal site, we can reuse the variable's value (to save sampling, and to keep good values), but we must
                // re-calculate the density because the distribution's parameters might have changed.
                // This is currently conservative - it assumes dependence on the proposal site more often than neccessary. We could decrease it with
                // a data dependency graph.
                let (val, log_weight) = if self.reached_proposal_site {
                    (val.clone(), dist.log_pdf(&val)?)
                } else {
                    (val.clone(), *log_weight)
                };

                self.proposal.reused_log_weight += log_weight;

                (val, log_weight)
            } else {
                // the variable wasn't in the previous run.
                let val = dist.sample()?;
                let log_weight = dist.log_pdf(&val)?;
                (val, log_weight)
            }
        } else {
            // there was no previous run
            let val = dist.sample()?;
            let log_weight = dist.log_pdf(&val)?;
            (val, log_weight)
        };

        self.proposal.trace.insert(key, (val.clone(), log_weight));

        Ok(val)
    }

    fn observe(
        &mut self,
        dist: &dyn Distribution,
        val: Value,
        _observe_number: Option<usize>,
    ) -> Result<Value, RuntimeError> {
        let log_weight = dist.log_pdf(&val)?;
        self.proposal.observed_log_weight += log_weight;
        Ok(val)
    }

    fn finish_one_evaluation(&mut self, val: Value) {
        // ratio of selecting x as the random variable

        // if there was a previous trace, do accept step
        // if accept succeeds,
        let accept = if let Some(last) = &self.last {
            let proposal = &self.proposal;
            // Introduction to PPL equation 4.21 (as of the version of the book in the repo in commit 105ee07cea1b61d83fcc0898cf9c5cce767bb9c0)
            let log_domain_prev = (last.trace.len() as f64).ln(); // probability we chose x as our single site
            let log_domain_proposal = (proposal.trace.len() as f64).ln(); // reverse probability we chose x site
            let log_acceptance_ratio =
                log_domain_prev + proposal.observed_log_weight + proposal.reused_log_weight
                    - log_domain_proposal
                    - last.observed_log_weight
                    - last.reused_log_weight;
            let acceptance_ratio = log_acceptance_ratio.exp();
            if acceptance_ratio >= 1_f64 || thread_rng().gen::<f64>() < acceptance_ratio {
                self.count += 1;
                if self.count >= self.skip {
                    self.count = 0;
                    self.samples.push(val);
                }

                true
            } else {
                false
            }
        } else {
            true
        };

        let fresh = RunMemory {
            trace: HashMap::new(),
            reused_log_weight: 0.,
            observed_log_weight: 0.,
        };
        let prev_proposal = std::mem::replace(&mut self.proposal, fresh);

        if accept {
            self.last.replace(prev_proposal);
        }

        // Choose new single site to change the sample. From book chapter 4.2: choose new x_0.
        let proposal_idx = thread_rng().gen_range(0..self.last.as_ref().unwrap().trace.len());
        self.proposal_site = *self
            .last
            .as_ref()
            .unwrap()
            .trace
            .keys()
            .nth(proposal_idx)
            .unwrap();
        self.reached_proposal_site = false;
        self.var_counter = 0;
    }

    fn finalize_and_make_dataset(self) -> Result<DataFile, RuntimeError> {
        let vals = flatten_to_numeric_vec_only(self.samples)?;

        Ok(DataFile {
            has_weights: false,
            data: vals,
        })
    }
}
