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

use std::path::PathBuf;

use clap::{AppSettings, Clap};
use lalrpop_util::lalrpop_mod;

use types::{RuntimeError, Value};

mod interpreter;

use interpreter::Interpreter;

lalrpop_mod!(pub grammar);

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(short, long, default_value = "10000")]
    n_samples: usize,

    file: PathBuf,
}
use serde::Serialize;
#[derive(Debug, Serialize)]
#[serde(untagged)]
enum ProgramResult {
    One(IntOrFloat),
    Many(Vec<IntOrFloat>),
}


#[derive(Debug, Serialize)]
#[serde(untagged)]
enum IntOrFloat {
    Int(i64),
    Float(f64),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();

    let file_stem = match &opts.file.file_stem() {
        Some(s) => *s,
        None => {
            eprintln!("Filename is not valid.");
            return Ok(());
        }
    };

    // Needs to be 'static, only because ParseError contains a reference and we want to return ParseError from main.
    let text: &'static str = Box::leak(std::fs::read_to_string(&opts.file)?.into_boxed_str());

    let parser = grammar::ProgramParser::new();
    let ast = parser.parse(&text)?;
    println!("{:#?}", ast);

    let mut interpreter = Interpreter::new();

    let values_result = match interpreter.eval_program(ast, opts.n_samples) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{:?}", e);
            return Ok(());
        }
    };

    let x = values_result
        .into_iter()
        .map(|v| match v {
            Value::Integer(i) => Ok(ProgramResult::One(IntOrFloat::Int(i))),
            Value::Float(f) => Ok(ProgramResult::One(IntOrFloat::Float(f))),
            Value::Vector(v) => {
                let this_v = v
                    .into_iter()
                    .map(|v| match v {
                        Value::Integer(i) => Ok(IntOrFloat::Int(i)),
                        Value::Float(f) => Ok(IntOrFloat::Float(f)),
                        _ => err!("Program should only return numbers or vecs of numbers."),
                    })
                    .collect::<Result<Vec<IntOrFloat>, RuntimeError>>();
                match this_v {
                    Ok(v) => Ok(ProgramResult::Many(v)),
                    Err(e) => Err(e),
                }
            }
            _ => err!("Program should only return numbers or vecs of numbers."),
        })
        .collect::<Result<Vec<ProgramResult>, RuntimeError>>();

    let x = match x {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{:?}", e);
            return Ok(());
        }
    };

    let data_json = serde_json::to_string(&x)?;

    let out_dir = std::path::Path::new("data/");
    let out_file = out_dir.join(file_stem).with_extension("json");

    std::fs::create_dir_all(out_dir)?;
    std::fs::write(out_file, data_json)?;

    Ok(())
}
