use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;
use macros::CastAny;
use crate::errors::{Error, ErrorId};
use crate::functions::CustomFunctionDef;
use crate::parser::nodes::{AssignExpr, BinExpr, CallExpr, ConstExpr, FunctionDefExpr, IdExpr, ListExpr, Node, NodeData, NoneExpr, PostfixExpr, Statement};
use crate::tokenizer::cursor::Range;
use crate::tokenizer::peeking_tokenizer::PeekingTokenizer;
use crate::tokenizer::token_type::TokenType;
use crate::resolver::scope::Scope;
use crate::resolver::unit::Unit;
use crate::tokenizer::Token;
use crate::tokenizer::token_type::TokenType::{Eq, EqDiv, EqMin, EqMult, EqPlus};

pub mod nodes;

pub struct CodeBlock {
    pub statements: Vec<Box<Statement>>,
    pub scope: Rc<RefCell<Scope>>,
}

impl<'a> CodeBlock {
    pub fn new(scope: RefCell<Scope>) -> Self {
        CodeBlock {
            scope: Rc::new(scope),
            statements: Vec::new(),
        }
    }
}

pub struct Parser<'a, 't> {
    tok: &'a mut PeekingTokenizer<'t>,
    errors: &'a mut Vec<Error>,
    statement_start: Range,
    code_block: CodeBlock,
}

impl<'a, 't> Into<(CodeBlock)> for Parser<'a, 't> {
    fn into(self) -> CodeBlock {
        self.code_block
    }
}

impl<'a, 't> Parser<'a, 't> {
    pub fn new (tok: &'a mut PeekingTokenizer<'t>, errors: &'a mut Vec<Error>, code_block: CodeBlock) -> Self {
        Parser {
            tok,
            errors,
            code_block,
            statement_start: Range { start: 0, end: 0, source_index: 0}
        }
    }

    pub fn parse(&mut self, for_block: bool) {
        while self.tok.peek().kind != TokenType::Eot {
            let stmt = self.parse_statement();
            self.code_block.statements.push(Box::new(stmt));
            if for_block && self.tok.peek().kind == TokenType::CurlClose {
                return;
            }
        }
    }

    fn parse_statement(&mut self) -> Statement {
        if let Some(stmt) = self.parse_function_def() {
            return stmt;
        }
        self.parse_expr_statement()
    }

    fn parse_function_def(&mut self) -> Option<Statement> {
        if !self.match_token(&TokenType::Function) {
            return None;
        };
        if self.tok.peek().kind != TokenType::Id {
            return Some(Statement::error(&mut self.errors, ErrorId::ExpectedId, self.tok.peek().clone(), ""));
        };
        let id = self.tok.next();

        if !self.match_token(&TokenType::ParOpen) {
            return Some(Statement::error(&mut self.errors, ErrorId::Expected, self.tok.peek().clone(), "("));
        };

        let mut param_defs: Vec<String> = Vec::new();

        while (self.tok.peek().kind == TokenType::Id) {
            let txt = self.code_block.scope.borrow().globals.get_text(&self.tok.next().range).to_string();
            param_defs.push(txt);
            if self.match_token(&TokenType::Comma) {
                continue;
            }
            if self.tok.peek().kind == TokenType::ParClose {
                break;
            }
            return Some(Statement::error(&mut self.errors, ErrorId::Expected, self.tok.peek().clone(), ",` or `)"));
        };

        if !self.match_token(&TokenType::ParClose) {
            return Some(Statement::error(&mut self.errors, ErrorId::Expected, self.tok.peek().clone(), ")"));
        };
        if !self.match_token(&TokenType::CurlOpen) {
            return Some(Statement::error(&mut self.errors, ErrorId::Expected, self.tok.peek().clone(), "{"));
        };
        let new_code_block = self.parse_block();
        let chars_left = self.tok.cur.chars.as_str().len();
        println!("chars after: {chars_left}");

        if !self.match_token(&TokenType::CurlClose) {
            return Some(Statement::error(&mut self.errors, ErrorId::Expected, self.tok.peek().clone(), "}"));
        };

        let mut fun_def_expr = FunctionDefExpr {
            id: self.get_text(&id.range).to_string(),
            id_range: id.range.clone(),
            arg_names: param_defs,
            node_data: NodeData { unit: Unit::none(), has_errors: false }
        };
        //TODO add range
        if(self.code_block.scope.borrow().local_function_defs.contains_key(&fun_def_expr.id)) {
            fun_def_expr.node_data.has_errors = true;
            self.errors.push(Error::build_1_arg(ErrorId::WFunctionOverride,id.range.clone(), &self.get_text(&id.range)));
        };

        self.code_block.scope.borrow_mut().add_local_function(new_code_block, &fun_def_expr);
        Some(Statement {
            node: Box::new(fun_def_expr),
            node_data: NodeData { unit: Unit::none(), has_errors: false }
        })
    }

    fn parse_block(&mut self) -> CodeBlock {
        let new_scope = Scope::copy_for_block(&self.code_block.scope);
        let new_code_block = CodeBlock::new(new_scope);
        let chars_left = self.tok.cur.chars.as_str().len();
        println!("chars before: {chars_left}");
        let mut parser = Parser::new(&mut self.tok, &mut self.errors, new_code_block);
        parser.parse(true);
        parser.into()
    }

    fn parse_expr_statement(&mut self) -> Statement {
        let mut stmt = Statement {
            node: self.parse_assign_expr(),
            node_data: NodeData {
                unit: Unit::none(),
                has_errors: false,
            }
        };
        match self.tok.peek().kind {
            TokenType::SemiColon => {
                self.tok.next();
            },
            TokenType::Eot => (),
            _ => {
                let t = self.tok.next(); //avoid dead loop!
                // let txt = self.get_text(&t.range); //WHY DOESN'T THIS WORK????
                self.errors.push(Error::build_1_arg(ErrorId::Expected, t.range.clone(), self.code_block.scope.borrow().globals.get_text(&t.range)));
                stmt.node_data.has_errors = true;
            }
        };
        stmt
    }

    fn parse_assign_expr(&mut self) -> Box<dyn Node> {
        if self.tok.peek().kind != TokenType::Id {
            return self.parse_add_expr();
        }
        use TokenType::*;
        let op_type = self.tok.peek_second().kind;
        let (Eq | EqPlus | EqMin | EqMult | EqDiv | EqUnit) = op_type else {
            return self.parse_add_expr();
        };

        let id = self.tok.next();
        if let Eq = op_type {
            self.tok.next(); //consume =
            let assign_expr = AssignExpr {
                node_data: NodeData { has_errors: false, unit: Unit::none() },
                id,
                expr: Parser::reduce_list(self.parse_list_expr())
            };
            //TODO: check EOT
            let txt = self.code_block.scope.borrow().globals.get_text(&assign_expr.id.range).to_string();
            self.code_block.scope.borrow_mut().var_defs.insert(txt);
            return Box::new(assign_expr);
        }
        unreachable!("TODO: implement else")
    }

    fn parse_add_expr(&mut self) -> Box<dyn Node> {
        let expr1 = self.parse_mult_expr();
        match self.tok.peek().kind {
            TokenType::Plus | TokenType::Min => {
                let op = self.tok.next().clone();
                let expr2 = self.parse_mult_expr();
                Box::new(BinExpr { expr1, op, expr2, node_data: NodeData { unit: Unit::none(), has_errors: false, }, implicit_mult: false })
            },
            _ => expr1
        }
    }

    fn parse_mult_expr(&mut self) -> Box<dyn Node> {
        let expr1 = self.parse_postfix_expr();
        match self.tok.peek().kind {
            TokenType::Mult | TokenType::Div | TokenType::Percent | TokenType::Modulo => {
                let op = self.tok.next().clone();
                let expr2 = self.parse_postfix_expr();
                Box::new(BinExpr { expr1, op, expr2, node_data: NodeData { unit: Unit::none(), has_errors: false, }, implicit_mult: false })
            },
            _ => expr1
        }
    }

    fn parse_postfix_expr(&mut self) -> Box<dyn Node> {
        let mut expr = self.parse_unit_expr();
        loop {
            match self.tok.peek().kind {
                TokenType::Dot | TokenType::Dec | TokenType::Inc | TokenType::Exclam => {
                    expr = self.parse_one_postfix(expr);
                },
                _ => break
            }
        }
        expr
    }

    fn parse_one_postfix(&mut self, node: Box<dyn Node>) -> Box<dyn Node> {
        match self.tok.peek().kind {
            TokenType::Dot => {
                self.tok.next();
                let t = self.tok.peek();
                let t_type = &t.kind.clone();
                let mut postfix = PostfixExpr { postfix_id: t.clone(), node, node_data: NodeData{unit: Unit::none(),has_errors: false,} };
                if t_type == &TokenType::Id {
                    self.tok.next();
                    // postfix.postfix_id already set!
                } else {
                    postfix.postfix_id = Token { kind: TokenType::Nullptr, range : Range { start: 0, end: 0, source_index: 0},
                        #[cfg(debug_assertions)]
                        text: "".to_string()
                    } //TODO: since range can be empty: use Option?
                }
                Box::new(postfix)
            },
            _ => node
        }
    }

    // if an id is 'glued' to a primary expr, without a dot in between, it should be a unit.
    fn parse_unit_expr(&mut self) -> Box<dyn Node> {
        let mut expr = self.parse_primary_expr();
        match self.tok.peek().kind {
            TokenType::Id => {
            //TODO: check if it's an existing var, in which case we'll ignore it as it's probably an implicit mult.
                let id = self.tok.next();
                let id= self.get_text(&id.range).to_string();
                let it = expr.deref_mut();
                let nd = &mut it.get_node_data();
                nd.unit.id = id;
            },
            _ => () //TODO: perhaps not use a match statement.
        }
        expr
    }

    fn match_token(&mut self, token_type: &TokenType) -> bool {
        if &self.tok.peek().kind != token_type {
            return false;
        };
        self.tok.next();
        true
    }

    #[inline]
    fn get_text(&self, range: &Range) -> String {
        self.code_block.scope.borrow().globals.get_text(&range).to_string()
    }

    fn parse_call_expr(&mut self, function_name: Token) -> Box<dyn Node> {
        let func_name_str = self.get_text(&function_name.range);
        if TokenType::ParOpen != self.tok.peek().kind {
            let error = Error::build_1_arg(ErrorId::FuncNoOpenPar, function_name.range.clone(), &func_name_str);
            self.errors.push(error);
            return Box::new(NoneExpr{node_data: NodeData { unit: Unit::none(), has_errors: true}, token: function_name.clone()});
        }
        self.tok.next();// eat `(`
        let args = self.parse_list_expr();
        //first argument may be NONE, with a token EOT, which is an invalid argument list in this case.
        //TODO: a function call with no parameters.
        let list_expr = args.as_any().downcast_ref::<ListExpr>().unwrap();
        if list_expr.nodes.len() == 1 {
            if let Some(none_expr) = list_expr.nodes.first().unwrap().as_any().downcast_ref::<NoneExpr>() {
                if none_expr.token.kind == TokenType::Eot {
                    let error = Error::build_1_arg(ErrorId::Eos, function_name.range.clone(), &self.get_text(&function_name.range));
                    self.errors.push(error);
                    return Box::new(NoneExpr{node_data: NodeData { unit: Unit::none(), has_errors: true}, token: function_name});
                }
            }
        }
        if !self.match_token(&TokenType::ParClose) {
            self.add_error(ErrorId::Expected, self.tok.peek().range.clone(), ")");
        }
        Box::new(CallExpr {
            node_data: NodeData { unit: Unit::none(), has_errors: false },
            function_name: func_name_str.to_string(),
            function_name_range: function_name.range.clone(),
            arguments: args
        })
    }

    fn add_error(&mut self, id: ErrorId, range: Range, arg1: &str) {
        let error = Error::build_1_arg(id, range, arg1);
        self.errors.push(error);
    }

    fn parse_primary_expr(&mut self) -> Box<dyn Node> {
        match self.tok.peek().kind {
            TokenType::Number => self.parse_number_expr(),
            TokenType::Id => {
                let t = self.tok.next();
                // let id = self.get_text(&t.range);
                let id = self.get_text(&t.range);
                if self.code_block.scope.borrow().local_function_defs.contains_key(&id) {
                    return self.parse_call_expr(t);
                } else {
                    if self.code_block.scope.borrow().globals.global_function_defs.contains_key(&id) {
                        return self.parse_call_expr(t);
                    }
                }
                Box::new(IdExpr {
                    id: t,
                    node_data: NodeData {  unit: Unit::none(), has_errors: false }
                })
            }
            TokenType::ParOpen => {
                self.tok.next();
                let mut expr = Parser::reduce_list(self.parse_list_expr());
                if !self.match_token(&TokenType::ParClose) {
                    let range = Range { start: 0, end: 0, source_index: 0};
                    let error = Error::build_1_arg(ErrorId::Expected, range, ")");
                    self.errors.push(error);
                }
                if expr.as_any().type_id() == TypeId::of::<BinExpr>() {
                    let bin_expr = expr.as_any_mut().downcast_mut::<BinExpr>().unwrap();
                    bin_expr.implicit_mult = false;
                }
                expr
            },
            _ => Box::new(NoneExpr { node_data: NodeData { unit: Unit::none(), has_errors: false,}, token: self.tok.peek().clone()})
        }
    }

    fn parse_number_expr(&mut self) -> Box<dyn Node> {
        //assuming type of token already checked.
        self.tok.next();
        Box::new(ConstExpr { value: self.tok.get_number(), node_data: NodeData { unit: Unit::none(),has_errors: false,}})
    }

    //TODO: try if this works with:
    // reduce_list(mut node: Box<dyn ListExpr>)...
    fn reduce_list(mut node: Box<dyn Node>) -> Box<dyn Node> {
        let list_expr = node.as_any_mut().downcast_mut::<ListExpr>().unwrap();
        if list_expr.nodes.len() == 1 {
            return list_expr.nodes.remove(0);
        }
        node
    }

    fn parse_list_expr(&mut self) -> Box<dyn Node> {
        let mut list_expr = ListExpr { nodes: Vec::new(), node_data: NodeData {unit: Unit::none(), has_errors: false}};
        loop {
            let expr = self.parse_add_expr();
            list_expr.nodes.push(expr);
            if list_expr.nodes.last().unwrap().as_any().is::<NoneExpr>() {
                break;
            }
            if self.tok.peek().kind != TokenType::Comma {
                break;
            }
            self.tok.next();
        }
        Box::new(list_expr)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use crate::errors::Error;
    use crate::parser::{CodeBlock, Parser};
    use crate::parser::nodes::{BinExpr, ConstExpr};
    use crate::resolver::globals::Globals;
    use crate::resolver::scope::Scope;
    use crate::tokenizer::peeking_tokenizer::PeekingTokenizer;
    use crate::tokenizer::token_type::TokenType;

    #[test]
    fn test_math_expression() {
        let txt = "2 + 3 * 4";
        let mut tok = PeekingTokenizer::new(txt);
        let mut globals = Globals::new();
        let mut scope = RefCell::new(Scope::new(Rc::new(globals)));
        let mut code_block = CodeBlock::new(scope);
        let mut errors = Vec::<Error>::new();
        let mut parser = Parser::new(&mut tok, &mut errors, code_block);
        parser.parse(false);
        let code_block: CodeBlock = parser.into();
        let stmt = code_block.statements.first().expect("There should be a statement here.");
        let root = stmt.node.as_any().downcast_ref::<BinExpr>().expect("There should be a BinExpr here.");
        let expr1 = root.expr1.as_any().downcast_ref::<ConstExpr>().expect("There should be a ConstExpr here.");
        assert_eq!(root.op.kind, TokenType::Plus);
        let expr2 = root.expr2.as_any().downcast_ref::<BinExpr>().expect("There should be a BinExpr here.");
        let expr2_1 = expr2.expr1.as_any().downcast_ref::<ConstExpr>().expect("There should be a ConstExpr here.");
        assert_eq!(expr2.op.kind, TokenType::Mult);
        let expr2_2 = expr2.expr2.as_any().downcast_ref::<ConstExpr>().expect("There should be a ConstExpr here.");
    }
}
