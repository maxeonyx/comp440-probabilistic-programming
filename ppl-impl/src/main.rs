mod ast;
mod types;

macro_rules! err {
    ($fstr:literal $(, $e:expr)*) => {{
        use crate::types::RuntimeError;
        Err(RuntimeError::new(format!($fstr, $($e,)*)))
    }};
}

type EvalResult = Result<Value, RuntimeError>;

mod functions;

use std::{ffi::OsStr, path::PathBuf, str::FromStr};

use ast::Program;
use clap::{AppSettings, Clap};
use lalrpop_util::lalrpop_mod;

use types::{RuntimeError, Value};

#[allow(unused)]
mod ancestral_sampler;
mod interpreter;
mod inference;
mod distributions;

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
        #[clap(arg_enum, short, long)]
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

use crate::inference::{InferenceAlg, PriorOnly};

#[derive(Debug, Serialize)]
pub struct DataFile {
    pub has_weights: bool,
    pub data: Vec<ProgramResult>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ProgramResult {
    One(IntOrFloat),
    Many(Vec<ProgramResult>),
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum IntOrFloat {
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

    Ok(match opts.cmd {
        Command::EvalOnce { file } => eval_once(program, file),
        Command::PriorOnly { n_samples, file } => {
            infer(program, file, PriorOnly::new())
        }
        Command::Infer { alg, file } => {

            let mut alg = match alg {
                Alg::LikelihoodWeighting => inference::LikelihoodWeighting::new(),
            };
            
            infer(program, file, alg)
        },
        Command::AncestralSample { .. } => unimplemented!("Inference not implemented yet."),
    }?)

}

fn file_name(opts: &Opts) -> &PathBuf {
    match &opts.cmd {
        Command::EvalOnce { file, .. } => file,
        Command::PriorOnly { file, .. } => file,
        Command::Infer { file, .. } => file,
        Command::AncestralSample { file, ..} => file,
    }
}

fn file_stem(file_name: &PathBuf) -> Option<&OsStr> {
    file_name.file_stem()
}

fn infer<T: InferenceAlg>(program: Program, file: PathBuf, n_samples: usize, mut alg: T) -> Result<(), Box<dyn std::error::Error>> {

    let mut interpreter = Interpreter::new(&mut alg);

    match interpreter.eval_program(program, 1) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{:?}", e);
            return Ok(());
        }
    };
    
    let data = match alg.finalize_and_write() {
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

    todo!()
}

fn eval_once(program: Program, _file: PathBuf) -> Result<(), Box<dyn std::error::Error>> {

    let mut alg = PriorOnly::new();
    let mut interpreter = Interpreter::new(&mut alg);

    match interpreter.eval_program(program, 1) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{:?}", e);
            return Ok(());
        }
    };
    
    let data = match alg.finalize_and_write() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{:?}", e);
            return Ok(());
        }
    };

    println!("{:#?}", data.data[0]);

    Ok(())
}
