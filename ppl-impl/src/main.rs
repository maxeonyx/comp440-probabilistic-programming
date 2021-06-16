macro_rules! err {
    ($fstr:literal $(, $e:expr)*) => {{
        use crate::types::RuntimeError;
        Err(RuntimeError::new(format!($fstr, $($e,)*)))
    }};
}

type EvalResult = Result<Value, RuntimeError>;

mod ast;
mod functions;
mod types;

use std::{ffi::OsStr, path::{Path, PathBuf}};

use ast::Program;
use clap::{AppSettings, Clap};
use inference::likelihood_weighting::LikelihoodWeighting;
use lalrpop_util::lalrpop_mod;

use types::{RuntimeError, Value};

#[allow(unused)]
mod ancestral_sampler;
mod distributions;
mod inference;
mod interpreter;

use interpreter::Interpreter;

lalrpop_mod!(#[allow(clippy::all)] pub grammar);

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Clap, PartialEq, Debug)]
enum Alg {
    LikelihoodWeighting,
    SingleSiteMetropolis {
        #[clap(short, long, default_value = "20")]
        skip: usize,
    },
}

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
enum Command {
    PriorOnly {
        #[clap(short, long, default_value = "10000")]
        n_samples: usize,
        file: PathBuf,
    },
    Infer {
        #[clap(short, long, default_value = "10000")]
        n_samples: usize,
        #[clap(subcommand)]
        alg: Alg,
        file: PathBuf,
    },
    EvalOnce {
        file: PathBuf,
    },
    AncestralSample {
        file: PathBuf,
    },
}

use serde::Serialize;

use crate::inference::{
    prior_only::PriorOnly, single_site_metropolis::SingleSiteMetropolis, InferenceAlg,
};

#[derive(Debug, Serialize)]
pub struct DataFile {
    pub has_weights: bool,
    pub data: Vec<ProgramResult>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ProgramResult {
    One(ResultValue),
    Many(Vec<ProgramResult>),
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ResultValue {
    Boolean(bool),
    Int(i64),
    Float(f64),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();

    let file_name = file_name(&opts);
    let _file_stem = match file_stem(file_name) {
        Some(s) => s,
        None => {
            eprintln!("Filename is not valid.");
            return Ok(());
        }
    };

    // Needs to be 'static, only because ParseError contains a reference and we want to return ParseError from main.
    let text: &'static str = Box::leak(std::fs::read_to_string(file_name)?.into_boxed_str());

    let parser = grammar::ProgramParser::new();
    let program = parser.parse(&text)?;
    println!("{:#?}", program);

    match opts.cmd {
        Command::EvalOnce { file } => eval_once(program, &file),
        Command::PriorOnly { n_samples, file } => infer(program, &file, n_samples, PriorOnly::new()),
        Command::Infer {
            alg,
            file,
            n_samples,
        } => match alg {
            Alg::LikelihoodWeighting => infer(program, &file, n_samples, LikelihoodWeighting::new()),
            Alg::SingleSiteMetropolis { skip } => {
                infer(program, &file, n_samples, SingleSiteMetropolis::new(skip))
            }
        },
        Command::AncestralSample { .. } => unimplemented!("Inference not implemented yet."),
    }
}

fn file_name(opts: &Opts) -> &Path {
    match &opts.cmd {
        Command::EvalOnce { file, .. } => file,
        Command::PriorOnly { file, .. } => file,
        Command::Infer { file, .. } => file,
        Command::AncestralSample { file, .. } => file,
    }
}

fn file_stem(file_name: &Path) -> Option<&OsStr> {
    file_name.file_stem()
}

fn infer<T: InferenceAlg>(
    program: Program,
    file: &Path,
    n_samples: usize,
    mut alg: T,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new(&mut alg);

    match interpreter.eval_program(program, n_samples) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{:?}", e);
            return Ok(());
        }
    };

    let data = match alg.finalize_and_make_dataset() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{:?}", e);
            return Ok(());
        }
    };

    let data_json = serde_json::to_string(&data)?;

    let out_dir = std::path::Path::new("data/");
    let file_stem = file_stem(&file).unwrap();
    let out_file = out_dir.join(file_stem).with_extension("json");

    std::fs::create_dir_all(out_dir)?;
    std::fs::write(out_file, data_json)?;

    Ok(())
}

fn eval_once(program: Program, _file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut alg = PriorOnly::new();
    let mut interpreter = Interpreter::new(&mut alg);

    match interpreter.eval_program(program, 1) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{:?}", e);
            return Ok(());
        }
    };

    let data = match alg.finalize_and_make_dataset() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{:?}", e);
            return Ok(());
        }
    };

    println!("{:#?}", data.data[0]);

    Ok(())
}
