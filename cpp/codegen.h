#ifndef CODEGEN_H_
#define CODEGEN_H_

#include <llvm/IR/LLVMContext.h>
#include <llvm/IR/Module.h>
#include <llvm/IR/Function.h>

#include "ast.h"
#include "llvm/IR/Function.h"

namespace client::codegen {

  void codegen(llvm::LLVMContext& context, llvm::Function* function, client::ast::block& block);


  llvm::Function* codegen(llvm::LLVMContext &context,
			  llvm::Module* module,
			  client::ast::function &function) {
  
    auto integer_type = llvm::IntegerType::get(context, 64);
    std::vector<llvm::Type*> args_type;
    for(auto& a : function.args) {
      args_type.emplace_back(integer_type);
    }
      
    auto function_type = llvm::FunctionType::get(integer_type, args_type, false);
    auto llvm_function = llvm::Function::Create(function_type,
						llvm::GlobalValue::CommonLinkage,
						function.name,
						module);
      
    codegen(context, llvm_function, function.body);
    return llvm_function;
  }

  std::unique_ptr<llvm::Module> codegen(llvm::LLVMContext &context,
					client::ast::program &program) {
    auto module = std::make_unique<llvm::Module>("my_module", context);
    for(auto& f : program.functions) {
      codegen(context, module.get(), f);
    }
    return module;
  }

  llvm::Value* codegen(llvm::LLVMContext& context, client::ast::expr& exp);  
  class codegen_visitor : public boost::static_visitor<llvm::Value*>
  {
  public:
  codegen_visitor(llvm::LLVMContext& context) : context_{context} {}
    llvm::LLVMContext& context_;
    llvm::Value* operator()(client::ast::var& value) const {
      // value.value

      return nullptr;
    }
    llvm::Value* operator()(client::ast::const_int& value) const {
      auto integer_type = llvm::IntegerType::get(context_, 64);      
      auto const_int = llvm::ConstantInt::get(integer_type, value.value, true);
      return const_int;
    }
    llvm::Value* operator()(client::ast::expr& value) const {
      return codegen(context_, value);
    }
  };

  llvm::Value* codegen(llvm::LLVMContext& context, client::ast::atom& atom) {
    return atom.apply_visitor(codegen_visitor(context));
  }

  llvm::Value* codegen(llvm::LLVMContext& context, client::ast::operation& op) {
    return nullptr;
  }

  llvm::Value* codegen(llvm::LLVMContext& context, client::ast::expr& exp) {
    auto first_value = codegen(context, exp.first);
    for(auto& oper : exp.rest) {
      codegen(context, oper);
    }
    return first_value;
  }

  void codegen(llvm::LLVMContext& context, llvm::Function* function, client::ast::block& block) {
    auto my_block = llvm::BasicBlock::Create(context, "entry", function);
    auto ret_value = codegen(context, block.ret);
    auto ret_inst = llvm::ReturnInst::Create(context, ret_value);
      
  }

  
  

}

#endif
