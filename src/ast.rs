use std::collections::HashMap;

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::values::BasicMetadataValueEnum;
use inkwell::values::{FunctionValue, IntValue};
use inkwell::IntPredicate;

#[derive(Debug)]
pub struct Code {
    pub functions: Vec<Fn>,
}

impl Code {
    pub fn codegen<'ctx>(&self, cg: &mut CodeGen<'ctx>) {
        for f in &self.functions {
            f.codegen(cg).unwrap();
        }
    }
}

#[derive(Debug)]
pub struct Fn {
    pub name: String,
    pub args: Vec<String>,
    pub body: Block,
}

impl<'ctx> Fn {
    fn codegen(&self, cg: &mut CodeGen<'ctx>) -> anyhow::Result<()> {
        let i64_type = cg.context.i64_type();
        let args = self
            .args
            .iter()
            .map(|_arg| i64_type.into())
            .collect::<Vec<BasicMetadataTypeEnum<'ctx>>>();
        let fn_type = i64_type.fn_type(&args[..], false);
        let function = cg.module.add_function(&self.name, fn_type, None);

        cg.functions.insert(self.name.clone(), function);

        cg.function_args.clear();
        for (i, arg) in self.args.iter().enumerate() {
            let value = function.get_nth_param(i as u32).unwrap().into_int_value();
            cg.function_args.insert(arg.clone(), value);
        }
        self.body.codegen(cg, function)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Let {
    pub name: String,
    pub expr: Expr,
}

#[derive(Debug)]
pub struct Block {
    pub lets: Vec<Let>,
    pub ret: Expr,
}

impl Block {
    fn codegen<'ctx>(
        &self,
        cg: &mut CodeGen<'ctx>,
        function: FunctionValue<'ctx>,
    ) -> anyhow::Result<BasicBlock<'ctx>> {
        let basic_block = cg.context.append_basic_block(function, "entry");
        cg.builder.position_at_end(basic_block);
        let ret_value = expr_codegen(&self.ret, cg, function)?;
        cg.builder.build_return(Some(&ret_value)).unwrap();

        Ok(basic_block)
    }
}

fn expr_codegen<'ctx>(
    expr: &Expr,
    cg: &mut CodeGen<'ctx>,
    function: FunctionValue<'ctx>,
) -> anyhow::Result<IntValue<'ctx>> {
    match expr {
        Expr::Num(val) => Ok(cg.context.i64_type().const_int(*val, false)),
        Expr::Neg(op) => {
            let value = expr_codegen(op, cg, function)?;
            let value = cg.builder.build_int_neg(value, "neg")?;
            Ok(value)
        }
        Expr::Add(lhs, rhs) => {
            let lhs_value = expr_codegen(lhs, cg, function)?;
            let rhs_value = expr_codegen(rhs, cg, function)?;
            let value = cg
                .builder
                .build_int_add(lhs_value, rhs_value, "sum")
                .unwrap();
            Ok(value)
        }
        Expr::Sub(lhs, rhs) => {
            let lhs_value = expr_codegen(lhs, cg, function)?;
            let rhs_value = expr_codegen(rhs, cg, function)?;
            let value = cg
                .builder
                .build_int_sub(lhs_value, rhs_value, "sub")
                .unwrap();
            Ok(value)
        }
        Expr::Mul(lhs, rhs) => {
            let lhs_value = expr_codegen(lhs, cg, function)?;
            let rhs_value = expr_codegen(rhs, cg, function)?;
            let value = cg
                .builder
                .build_int_mul(lhs_value, rhs_value, "mul")
                .unwrap();
            Ok(value)
        }
        Expr::Div(lhs, rhs) => {
            let lhs_value = expr_codegen(lhs, cg, function)?;
            let rhs_value = expr_codegen(rhs, cg, function)?;
            let value = cg
                .builder
                .build_int_signed_div(lhs_value, rhs_value, "sub")
                .unwrap();
            Ok(value)
        }
        Expr::Bigger(lhs, rhs) => {
            let lhs_value = expr_codegen(lhs, cg, function)?;
            let rhs_value = expr_codegen(rhs, cg, function)?;
            let value = cg
                .builder
                .build_int_compare(IntPredicate::SGT, lhs_value, rhs_value, "bigger")
                .unwrap();
            Ok(value)
        }
        Expr::Smaller(lhs, rhs) => {
            let lhs_value = expr_codegen(lhs, cg, function)?;
            let rhs_value = expr_codegen(rhs, cg, function)?;
            let value = cg
                .builder
                .build_int_compare(IntPredicate::SLT, lhs_value, rhs_value, "smaller")
                .unwrap();
            Ok(value)
        }
        Expr::Var(name) => Ok(cg.function_args.get(name).unwrap().clone()),
        Expr::Call(name, exprs) => {
            let function = cg.functions.get(name).unwrap().clone();
            let args: Vec<BasicMetadataValueEnum<'ctx>> = exprs
                .iter()
                .map(|expr| expr_codegen(expr, cg, function).unwrap().into())
                .collect();
            Ok(cg
                .builder
                .build_call(function, &args[..], "call")
                .unwrap()
                .try_as_basic_value()
                .left()
                .unwrap()
                .into_int_value())
        }
        Expr::If(cond, a, b) => {
            let cond_val = expr_codegen(cond, cg, function)?;

            let then_block = cg.context.append_basic_block(function, "then");
            let else_block = cg.context.append_basic_block(function, "else");
            let merge_block = cg.context.append_basic_block(function, "merge");

            // let zero = cg.context.i32_type().const_int(0, true);
            // let cond_val = cg.builder.build_int_compare(IntPredicate::NE, cond_val, zero, "cond_val")?;
            let _cond_branch = cg
                .builder
                .build_conditional_branch(cond_val, then_block, else_block)
                .unwrap();

            cg.builder.position_at_end(then_block);
            let then_val = expr_codegen(a, cg, function)?;
            let _br = cg.builder.build_unconditional_branch(merge_block);

            cg.builder.position_at_end(else_block);
            let else_val = expr_codegen(b, cg, function)?;
            let _br = cg.builder.build_unconditional_branch(merge_block);

            cg.builder.position_at_end(merge_block);
            let phi = cg.builder.build_phi(cg.context.i64_type(), "phi")?;

            use inkwell::values::BasicValue;
            let mut incoming = Vec::<(&dyn BasicValue<'ctx>, BasicBlock<'ctx>)>::new();
            incoming.push((&then_val, then_block));
            phi.add_incoming(&incoming);

            let mut incoming = Vec::<(&dyn BasicValue<'ctx>, BasicBlock<'ctx>)>::new();
            incoming.push((&else_val, else_block));
            phi.add_incoming(&incoming);

            Ok(phi.as_basic_value().into_int_value())
        }
        // _ => todo!(),
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Num(u64),
    Var(String),

    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),

    Bigger(Box<Expr>, Box<Expr>),
    Smaller(Box<Expr>, Box<Expr>),

    Call(String, Vec<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
}

pub struct CodeGen<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub execution_engine: ExecutionEngine<'ctx>,
    pub function_args: HashMap<String, IntValue<'ctx>>,
    pub functions: HashMap<String, FunctionValue<'ctx>>,
}

type MainFunc = unsafe extern "C" fn() -> u64;

impl<'ctx> CodeGen<'ctx> {
    pub fn compile(&self) -> Option<JitFunction<MainFunc>> {
        unsafe { self.execution_engine.get_function("main").ok() }
    }
}
