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

use clap::{AppSettings, Clap};
use lalrpop_util::lalrpop_mod;
use plotters::style::RGBAColor;
use types::{RuntimeError, Value};

mod interpreter;

use interpreter::Interpreter;

lalrpop_mod!(pub grammar);

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(short, long, default_value = "10000")]
    n_samples: u64,

    filename: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();
    
    // Needs to be 'static, only because ParseError contains a reference and we want to return ParseError from main.
    let text: &'static str = Box::leak(std::fs::read_to_string(&opts.filename)?.into_boxed_str());

    let parser = grammar::ProgramParser::new();
    let ast = parser.parse(&text)?;
    println!("{:#?}", ast);

    let mut interpreter = Interpreter::new();
    let samples = (0..opts.n_samples).try_fold(Vec::new(), |mut samples, _i| {
        let val = interpreter.eval(&ast.expression)?;
        match val {
            Value::Float(v) => samples.push(v),
            _ => return err!("Only float return values supported right now."),
        };
        Ok(samples)
    });

    let samples = match samples {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{:?}", e);
            return Ok(());
        }
    };

    use plotters::prelude::*;
    let image_filename = format!("{}.png", &opts.filename);
    let root = BitMapBackend::new(&image_filename, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption(&image_filename, ("sans-serif", 20).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(-10f32..10f32, 0f32..2f32)?;
    
    chart.configure_mesh().draw()?;
    let color = RGBAColor::from((0u8, 150u8, 255u8, 64u8));
    chart
    .draw_series(plotters::series::PointSeries::of_element(samples.iter().map(|v| (*v as f32, 1.0f32)), 3, &RED, &|coord, size, style| {
        plotters::element::BitMapElement::new(coord, (size, size))
    }))?
    ;

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    Ok(())
}
