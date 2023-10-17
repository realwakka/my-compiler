use std::collections::HashMap;

mod ast;

use inkwell::context::Context;
use inkwell::OptimizationLevel;

use chumsky::prelude::*;


fn code_parser() -> impl Parser<char, ast::Code, Error = Simple<char>> {
    let ident = text::ident().padded();
    let args = 
	just('(').ignore_then(
	    ident
		.then(just(',').ignore_then(ident).repeated())
		.map(|(s0, mut s)| {
		    s.push(s0);
		    s
		}))
	.then_ignore(just(')'))
	.or(just("()").map(|_| Vec::new()));

    let expr = recursive(|expr| {
	let var = ident.map(|s| ast::Expr::Var(s)).padded();

	let int = text::int(10)
            .map(|s: String| ast::Expr::Num(s.parse().unwrap()))
            .padded();

	let atom = int.or(var)
            .or(expr.clone().delimited_by(just('('), just(')'))).padded();

	let op = |c| just(c).padded();

	let unary = op('-')
            .repeated()
            .then(atom)
            .foldr(|_op, rhs| ast::Expr::Neg(Box::new(rhs)));

	let product = unary.clone()
            .then(op('*').to(ast::Expr::Mul as fn(_, _) -> _)
		  .or(op('/').to(ast::Expr::Div as fn(_, _) -> _))
		  .then(unary)
		  .repeated())
            .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

	let sum = product.clone()
            .then(op('+').to(ast::Expr::Add as fn(_, _) -> _)
		  .or(op('-').to(ast::Expr::Sub as fn(_, _) -> _))
		  .then(product)
		  .repeated())
            .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

	let bigger = sum.clone()
            .then(op('>').to(ast::Expr::Bigger as fn(_, _) -> _)
		  .or(op('<').to(ast::Expr::Smaller as fn(_, _) -> _))
		  .then(sum.clone())
		  .repeated())
            .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

	let call = ident
	    .then_ignore(just("()"))
	    .map(|name| ast::Expr::Call(name, Vec::new())).padded();

	let call_with_args =
	    ident.then_ignore(just('('))
	    .then(expr.clone())
	    .then(just(',').padded().ignore_then(expr.clone()).repeated())
	    .then_ignore(just(')')).padded()
	    .map(|((name, e), mut es)| {
		es.push(e);
		ast::Expr::Call(name, es)
	    });
	let r#if =
	    just("if").padded()
	    .ignore_then(expr.clone()).padded()
	    .then_ignore(just('{')).padded()
	    .then(expr.clone()).padded()
	    .then_ignore(just("}")).padded()
	    .then_ignore(just("else")).padded()
	    .then_ignore(just("{")).padded()
	    .then(expr.clone()).padded()
	    .then_ignore(just("}")).padded()
	    .map(|((cond, t), f)| ast::Expr::If(Box::new(cond), Box::new(t), Box::new(f)));

	// r#if.or(bigger).or(call).or(call_with_args)
	// call.or(call_with_args.or(sum)).or(r#if)
	// r#if.or(bigger).or(call).or(call_with_args);
	// bigger.or(call_with_args)
	call_with_args.or(r#if).or(bigger)
	// call_with_args.or(bigger)
    });


    let r#let = text::keyword("let")
	.ignore_then(ident)
	.then_ignore(just('='))
	.then(expr.clone())
	.then_ignore(just(';'))
	.padded()
	.map(|e| ast::Let { name: e.0.clone(), expr: e.1.clone() }).padded();

    let fn_body = just('{')
        // .ignore_then(r#let.repeated())
	.then(expr)
	.then_ignore(just('}')).padded().map(|(lets, expr)| ast::Block{ lets: Vec::new(), ret: expr } );

    let r#fn = text::keyword("fn")
        .ignore_then(ident)
        .then(args)
        .then(fn_body)
        .map(|((name, args), body)| ast::Fn {
            name,
            args,
	    body,
        });

    let code = r#fn.padded().repeated().map(|f| ast::Code { functions: f });
    code
}

// struct CodeGen<'ctx> {
//     context: &'ctx Context,
//     module: Module<'ctx>,
//     builder: Builder<'ctx>,
//     execution_engine: ExecutionEngine<'ctx>,
//     function_args: HashMap<String, IntValue<'ctx>>,
//     functions: HashMap<String, FunctionValue<'ctx>>,
// }


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

    let code = code_parser().parse(src).unwrap();
    println!("{:?}", code);
    code.codegen(&mut codegen);

    let main_fn = codegen.compile().unwrap();
    unsafe {
	println!("main func returned: {:?}", main_fn.call());
    }

    Ok(())
}
