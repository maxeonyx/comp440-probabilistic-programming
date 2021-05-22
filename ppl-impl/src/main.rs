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
use plotters::{prelude::Path, style::RGBAColor};
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

    let values_result = interpreter.eval_program(ast, opts.n_samples);

    

    let samples = values_result

    use itertools::Itertools;

    let samples = match samples {
        Ok(s) => {
            match val {
                Value::Float(v) => samples.push(v),
                _ => return err!("Only float return values supported right now."),
            };
        },
        Err(e) => {
            eprintln!("{:?}", e);
            return Ok(());
        }
    };

    let data_json = serde_json::to_string(&samples)?;

    let out_dir = std::path::Path::new("data/");
    let out_file = out_dir.join(file_stem).with_extension("json");

    std::fs::create_dir_all(out_dir)?;
    std::fs::write(out_file, data_json)?;

    Ok(())
}
