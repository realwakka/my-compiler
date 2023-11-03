#include <boost/spirit/home/x3.hpp>
#include <boost/fusion/include/adapt_struct.hpp>
#include <boost/fusion/include/io.hpp>
#include <boost/variant.hpp>
#include <boost/spirit/home/x3/support/ast/variant.hpp>

#include <llvm/IR/LLVMContext.h>
#include <llvm/IR/Module.h>

#include <iostream>
#include <string>

#include "ast.h"
#include "parser.h"
#include "codegen.h"

#include "llvm/IR/DerivedTypes.h"
#include "llvm/IR/Function.h"
#include "llvm/IR/Type.h"
#include "llvm/Support/raw_ostream.h"

int main() {
  std::string str = "fn add(x, y){1}";
  std::cout << str << std::endl;
  client::ast::program root;
  bool r = phrase_parse(str.begin(), str.end(), client::parser::program, x3::ascii::space, root);

  std::cout << root.print() << std::endl;

  llvm::LLVMContext context;
  auto mod = client::codegen::codegen(context, root);
  mod->print(llvm::errs(), nullptr);
  
  
  // llvm::Type type{context, llvm::Type::IntegerTyID};
  // auto function_type = llvm::FunctionType::get(&type, false);
  // llvm::Function::Create();
  // auto result = boost::apply_visitor( my_visitor(), root.value );
  

  // std::cout << boost::get<client::ast::var>(root.value).value <<std::endl;

  

  // std::cout <<  << std::endl;
	
  // compiler<std::string::iterator> comp;
  // bool r = phrase_parse(str.begin(), str.end(), comp, ascii::space);

  // boost::visit([](auto&& arg){ std::cout << arg.value; }, root.value);
  return 0;
}

