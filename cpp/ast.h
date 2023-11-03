#ifndef AST_H_
#define AST_H_

#include "llvm/ADT/APInt.h"
#include "llvm/IR/BasicBlock.h"
#include "llvm/IR/DerivedTypes.h"
#include "llvm/IR/Function.h"
#include "llvm/IR/GlobalValue.h"
#include "llvm/IR/Type.h"
#include "llvm/IR/Value.h"
#include "llvm/IR/Instructions.h"
#include <boost/spirit/home/x3.hpp>
#include <boost/fusion/include/adapt_struct.hpp>
#include <boost/fusion/include/io.hpp>
#include <boost/variant.hpp>
#include <boost/spirit/home/x3/support/ast/variant.hpp>

#include <llvm/IR/LLVMContext.h>
#include <llvm/IR/Module.h>
#include <llvm/IR/Constants.h>

#include <list>
#include <memory>

namespace x3 = boost::spirit::x3;

namespace client::ast {

  struct var {
    std::string value;
  };

  struct const_int {
    int64_t value;
  };

  class my_visitor;

  // struct nil {};
  struct expr;

  struct atom : x3::variant<const_int, var, x3::forward_ast<expr>> {
    using base_type::base_type;
    using base_type::operator=;
    std::string print();
  };

  struct operation {
    char oper;
    atom operand;
    std::string print() {
      std::string ret;
      ret += "operation {op: ";
      ret += oper;
      ret += " operand: " + operand.print();
      ret += "}";
      return ret;
    }
  };

  struct expr {
    atom first;
    std::list<operation> rest;

    std::string print() {
      std::stringstream ss;
      ss << "expr { first: " << first.print() << " rest : [";
      for(auto& op : this->rest) {
	ss << op.print() << ",";
      }

      ss << "]}";
      return ss.str();
    }

  };

  class my_visitor : public boost::static_visitor<std::string>
  {
  public:
    std::string operator()(client::ast::var& value) const
    {
      return value.value;
    }
    
    std::string operator()(client::ast::const_int& value) const
    {
      return std::to_string(value.value);
    }

    std::string operator()(client::ast::expr& value) const
    {
      return value.print();
    }
  };


  std::string atom::print() { return this->apply_visitor(my_visitor()); }


  struct block {
    expr ret;
    std::string print() {
      std::stringstream ss;
      ss << "block {" << ret.print() << "}";
      return ss.str();      
    }
  };

  struct function {
    std::string name;
    std::vector<std::string> args;
    block body;
    
    std::string print() {
      std::stringstream ss;
      ss << "function { name: " << name << ", args:[";

      for(auto& arg : args) {
	ss << arg << ",";
      }
      ss << "], body:"<<body.print() << "}";
      return ss.str();
    }
  };

  struct program {
    std::vector<function> functions;

    std::string print() {
      std::stringstream ss;
      ss << "program {functions: [";
      for(auto& f : functions) {
	ss << f.print() << ",";
      }
      ss << "]";
      return ss.str();
    }
  };
}

BOOST_FUSION_ADAPT_STRUCT(client::ast::var, value)
BOOST_FUSION_ADAPT_STRUCT(client::ast::const_int, value)

BOOST_FUSION_ADAPT_STRUCT(client::ast::operation, oper, operand)
BOOST_FUSION_ADAPT_STRUCT(client::ast::expr, first, rest)

BOOST_FUSION_ADAPT_STRUCT(client::ast::block, ret)
BOOST_FUSION_ADAPT_STRUCT(client::ast::function, name, args, body)
BOOST_FUSION_ADAPT_STRUCT(client::ast::program, functions)
  
#endif
