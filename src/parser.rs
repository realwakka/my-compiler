use chumsky::prelude::*;

use crate::ast;

fn expr_parser() -> impl Parser<char, ast::Expr, Error = Simple<char>> {
    let expr = recursive(|expr| {
        let ident = text::ident().padded();
        let var = ident.then_ignore(end()).map(|s| ast::Expr::Var(s));
        let int = text::int(10)
            .map(|s: String| ast::Expr::Num(s.parse().unwrap()))
            .padded();

        let call = text::ident()
            .padded()
            .then(
                expr.clone()
                    .separated_by(just(','))
                    .allow_trailing()
                    .delimited_by(just('('), just(')')),
            )
            .padded()
            .map(|(f, args)| ast::Expr::Call(f, args));

        let atom = int
            .or(var)
            .or(expr.clone().delimited_by(just('('), just(')')))
            .or(call)
            .or(ident.map(ast::Expr::Var));

        let op = |c| just(c).padded();

        let unary = op('-')
            .repeated()
            .then(atom)
            .foldr(|_op, rhs| ast::Expr::Neg(Box::new(rhs)));

        let product = unary
            .clone()
            .then(
                op('*')
                    .to(ast::Expr::Mul as fn(_, _) -> _)
                    .or(op('/').to(ast::Expr::Div as fn(_, _) -> _))
                    .then(unary)
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

        let sum = product
            .clone()
            .then(
                op('+')
                    .to(ast::Expr::Add as fn(_, _) -> _)
                    .or(op('-').to(ast::Expr::Sub as fn(_, _) -> _))
                    .then(product.clone())
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

        let bigger = sum
            .clone()
            .then(
                op('>')
                    .to(ast::Expr::Bigger as fn(_, _) -> _)
                    .or(op('<').to(ast::Expr::Smaller as fn(_, _) -> _))
                    .then(sum.clone())
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

        let r#if = just("if")
            .padded()
            .ignore_then(expr.clone())
            .padded()
            .then_ignore(just('{'))
            .padded()
            .then(expr.clone())
            .padded()
            .then_ignore(just("}"))
            .padded()
            .then_ignore(just("else"))
            .padded()
            .then_ignore(just("{"))
            .padded()
            .then(expr.clone())
            .padded()
            .then_ignore(just("}"))
            .padded()
            .map(|((cond, t), f)| ast::Expr::If(Box::new(cond), Box::new(t), Box::new(f)));

        r#if.or(bigger)
    });

    expr
}

#[test]
fn test_var() {
    let ident = text::ident::<char, Simple<char>>().padded();
    let var = ident.map(|s| ast::Expr::Var(s)).then_ignore(end());

    let call = text::ident()
        .padded()
        .then(
            var.clone()
                .separated_by(just(','))
                .allow_trailing()
                .delimited_by(just('('), just(')')),
        )
        .map(|(f, _args)| ast::Expr::Call(f, Vec::new()));

    let p = var.or(call);

    println!("{:?}", p.parse("asdf()"));
}

#[test]
fn test_expr_parser() {
    let parser = expr_parser();
    let expr = parser.parse("main()").unwrap();

    println!("{:?}", expr);
    assert!(matches!(expr, crate::ast::Expr::Call(_name, _args)));

    let expr = parser.parse("a() + b() + c()").unwrap();
    println!("{:?}", expr);
    assert!(matches!(expr, crate::ast::Expr::Add(_a, _b)));
}

pub fn code_parser() -> impl Parser<char, ast::Code, Error = Simple<char>> {
    let ident = text::ident().padded();
    let args = just('(')
        .ignore_then(
            ident
                .then(just(',').ignore_then(ident).repeated())
                .map(|(s0, mut s)| {
                    s.push(s0);
                    s
                }),
        )
        .then_ignore(just(')'))
        .or(just("()").map(|_| Vec::new()));

    let expr = expr_parser();

    // let r#let = text::keyword("let")
    // 	.ignore_then(ident)
    // 	.then_ignore(just('='))
    // 	.then(expr)
    // 	.then_ignore(just(';'))
    // 	.padded()
    // 	.map(|e| ast::Let { name: e.0.clone(), expr: e.1.clone() }).padded();

    let fn_body = just('{')
        // .ignore_then(r#let.repeated())
        .then(expr)
        .then_ignore(just('}'))
        .padded()
        .map(|(_lets, expr)| ast::Block {
            lets: Vec::new(),
            ret: expr,
        });

    let r#fn = text::keyword("fn")
        .ignore_then(ident)
        .then(args)
        .then(fn_body)
        .map(|((name, args), body)| ast::Fn { name, args, body });

    let code = r#fn.padded().repeated().map(|f| ast::Code { functions: f });
    code
}
