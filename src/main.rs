use std::env;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::process::exit;
use inkwell::values::AnyValue;
use crate::compiler::codegen::codegen;
use crate::compiler::lexer::lex;
use crate::compiler::parser::parse;

mod compiler;

fn main()
{
    let args: Vec<String> = env::args().collect();
    if args.len() != 3
    {
        println!("Usage: \n\
        brain_f_rs [input] [output]");
        exit(1);
    }
    let code = match read_to_string(&args[1])
    {
        Ok(val) => val,
        Err(_) =>
            {
                println!("Could not find the input file!");
                exit(1);
            }
    };
    let code = codegen(parse(lex(&code)).unwrap().into_iter());
    let mut file = File::create(&args[2]).map_err(|_|
        {
            println!("Could not create the output file!");
            exit(1);
        }).unwrap();
    file.write_all(&code).map_err(|_|
        {
            println!("Could not write to the output file!");
            exit(1);
        }).unwrap();
}
