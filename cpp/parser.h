#ifndef PARSER_H_
#define PARSER_H_

#include <boost/spirit/home/x3.hpp>
#include <boost/fusion/include/adapt_struct.hpp>

#include "ast.h"

namespace client::parser {
  namespace x3 = boost::spirit::x3;
  namespace ascii = boost::spirit::x3::ascii;

  using x3::int_;
  using x3::lit;
  using x3::double_;
  using x3::lexeme;
  using ascii::char_;

  x3::rule<class identifier, std::string> const identifier= "identifier";  
  x3::rule<class var, ast::var> const var = "var";
  x3::rule<class const_int, ast::const_int> const const_int = "const_int";
  x3::rule<class atom, ast::atom> const atom = "atom";

  x3::rule<class product, ast::operation> const product = "product";
  x3::rule<class expr, ast::expr> const expr = "expr";
  
  x3::rule<class addition, ast::operation> const addition = "addition";
  x3::rule<class add_expr, ast::expr> const add_expr = "add_expr";

  x3::rule<class block, ast::block> const block = "block";  
  x3::rule<class function, ast::function> const function = "function";

  x3::rule<class program, ast::program> const program = "program";

  auto const identifier_def = x3::lexeme[(x3::alpha | x3::char_('_')) >> *(x3::alnum | x3::char_('_'))];
  auto const var_def = identifier;
  auto const const_int_def = int_;
  auto const atom_def = const_int | var | expr;
  auto const product_def = (char_('*') | char_('/')) >> atom;
  auto const expr_def = atom >> *(product);

  auto const addition_def = (char_('+') | char_('-')) >> expr;
  auto const add_expr_def = atom >> *(addition);
  auto const block_def = lit('{') >> add_expr >> lit('}');
  auto const function_def = lit("fn") >> identifier >> lit('(') >> (identifier % ',') >> lit(')') >> block;
  auto const program_def = *(function);

  BOOST_SPIRIT_DEFINE(identifier, var, const_int, atom, product, addition, expr, add_expr, block, function, program);
} // namespace client::parser

#endif
