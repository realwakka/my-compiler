use std::collections::HashMap;

use chumsky::Parser;

mod ast;
mod parser;

use inkwell::context::Context;
use inkwell::OptimizationLevel;

fn main() -> anyhow::Result<()> {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();

    let context = Context::create();
    let module = context.create_module("sum");
    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).unwrap();
    let mut codegen = ast::CodeGen {
        context: &context,
        module,
        builder: context.create_builder(),
        execution_engine,
	function_args: HashMap::new(),
	functions: HashMap::new(),
    };

    let code = parser::code_parser().parse(src).unwrap();
    println!("{:?}", code);
    code.codegen(&mut codegen);

    codegen.module.print_to_file("test.ll").unwrap();

    let main_fn = codegen.compile().unwrap();
    unsafe {
	println!("main func returned: {:?}", main_fn.call());
    }

    Ok(())
}
