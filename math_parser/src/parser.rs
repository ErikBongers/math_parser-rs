use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use macros::CastAny;
use crate::errors::{Error, ErrorId};
use crate::parser::nodes::{AssignExpr, BinExpr, CallExpr, CommentExpr, ConstExpr, FunctionDefExpr, HasRange, IdExpr, ListExpr, Node, NodeData, NoneExpr, PostfixExpr, Statement, UnaryExpr, UnitExpr};
use crate::resolver::globals::Globals;
use crate::tokenizer::cursor::Range;
use crate::tokenizer::peeking_tokenizer::PeekingTokenizer;
use crate::tokenizer::token_type::TokenType;
use crate::resolver::scope::Scope;
use crate::resolver::unit::Unit;
use crate::tokenizer::Token;
use crate::tokenizer::token_type::TokenType::{Div, Eq, EqDiv, EqMin, EqMult, EqPlus};

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

pub struct Parser<'g, 'a, 't> {
    globals: &'g Globals,
    tok: &'a mut PeekingTokenizer<'t>,
    errors: &'a mut Vec<Error>,
    statement_start: Range,
    code_block: CodeBlock,
}

impl<'g, 'a, 't> Into<(CodeBlock)> for Parser<'g, 'a, 't> {
    fn into(self) -> CodeBlock {
        self.code_block
    }
}

impl<'g, 'a, 't> Parser<'g, 'a, 't> {
    pub fn new (globals: &'g Globals, tok: &'a mut PeekingTokenizer<'t>, errors: &'a mut Vec<Error>, code_block: CodeBlock) -> Self {
        Parser {
            globals,
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
        if let Some(stmt) = self.parse_echo_comment() {
            return stmt;
        }
        if let Some(stmt) = self.parse_function_def() {
            return stmt;
        }
        self.parse_expr_statement()
    }

    fn parse_echo_comment(&mut self) -> Option<Statement> {
        let token = self.tok.peek();
        if token.kind != TokenType::EchoCommentLine {
            return None;
        };
        Some( Statement {
            node_data: NodeData { unit: Unit::none(), has_errors: false },
            node: Box::new(CommentExpr { node_data: NodeData { unit: Unit::none(), has_errors: false }, token: self.tok.next() }),
        })
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
            let txt = self.globals.get_text(&self.tok.next().range).to_string();
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

        if !self.match_token(&TokenType::CurlClose) {
            return Some(Statement::error(&mut self.errors, ErrorId::Expected, self.tok.peek().clone(), "}"));
        };

        let mut fun_def_expr = FunctionDefExpr {
            id: self.globals.get_text(&id.range).to_string(),
            id_range: id.range.clone(),
            arg_names: param_defs,
            node_data: NodeData { unit: Unit::none(), has_errors: false }
        };
        //TODO add range
        if(self.code_block.scope.borrow().local_function_defs.contains_key(&fun_def_expr.id)) {
            fun_def_expr.node_data.has_errors = true;
            self.errors.push(Error::build(ErrorId::WFunctionOverride, id.range.clone(), &[&self.globals.get_text(&id.range)]));
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
        let mut parser = Parser::new(&self.globals, &mut self.tok, &mut self.errors, new_code_block);
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
                self.errors.push(Error::build(ErrorId::Expected, t.range.clone(), &[self.globals.get_text(&t.range)]));
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
            let txt = self.globals.get_text(&assign_expr.id.range).to_string();
            self.code_block.scope.borrow_mut().var_defs.insert(txt);
            return Box::new(assign_expr);
        }
        //build this expression: AssignExpr {id, BinExpr{ IdExpr, op, expr }}
        let eq_op = self.tok.next();
        let id_expr = IdExpr {
            node_data: NodeData::new(),
            id: id.clone(),
        };

        let expr : Box<dyn Node> = match op_type {
            EqPlus | EqMin | EqMult | EqDiv => {
                let bin_op = match op_type {
                    EqPlus => Plus,
                    EqMin => Min,
                    EqMult => Min,
                    EqDiv => Div,
                    _ => unreachable!()
                };
                Box::new(BinExpr {
                    node_data: NodeData::new(),
                    expr1: Box::new(id_expr),
                    op: Token {
                        kind: bin_op,
                        range: eq_op.range,
                        #[cfg(debug_assertions)]
                        text: "Eq_xxx".to_string(),
                    } ,
                    expr2: self.parse_add_expr(),
                    implicit_mult: false,
                })
            },
            EqUnit => {
                let unit_token = if self.tok.peek().kind == TokenType::Id { //assume id = a unit
                    self.tok.next()
                } else {
                    Token {
                        kind: TokenType::ClearUnit,
                        range: Range::none(),
                        #[cfg(debug_assertions)]
                        text: "".to_string(),
                    }
                };
                Box::new(PostfixExpr {
                    node_data: NodeData::new(),
                    node: Box::new(id_expr),
                    postfix_id: unit_token,
                })
            },
            _ => unreachable!("expected a Eq operator.")
        };

        let assign_expr = AssignExpr {
            node_data: NodeData::new(),
            id,
            expr,
        };
        Box::new(assign_expr)
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
        let mut expr1 = self.parse_power_expr();
        loop {
            match self.tok.peek().kind {
                TokenType::Mult | TokenType::Div | TokenType::Percent | TokenType::Modulo => {
                    let op = self.tok.next().clone();
                    let expr2 = self.parse_power_expr();
                    if op.kind == Div {
                        if expr2.is_implicit_mult() {
                            self.add_error(ErrorId::WDivImplMult, expr2.get_range().clone(), "");
                        }
                    }
                    expr1 = Box::new(BinExpr { expr1, op, expr2, node_data: NodeData { unit: Unit::none(), has_errors: false }, implicit_mult: false })
                }
                _ => break
            }
        };
        expr1
    }

    fn parse_power_expr(&mut self) -> Box<dyn Node> {
        let mut expr1 = self.parse_implicit_mult();
        loop {
            match self.tok.peek().kind {
                TokenType::Power => {
                    let op = self.tok.next().clone();
                    let expr2 = self.parse_power_expr(); //right associative!
                    let bin_expr = BinExpr { expr1, op, expr2, node_data: NodeData { unit: Unit::none(), has_errors: false }, implicit_mult: false };
                    if bin_expr.expr1.deref().is_implicit_mult() || bin_expr.expr2.deref().is_implicit_mult() {
                        self.add_error(ErrorId::WPowImplMult, bin_expr.get_range().clone(), "");
                    }
                    expr1 = Box::new(bin_expr);

                }
                _ => break
            }
        }
        expr1
    }

    fn parse_implicit_mult(&mut self) -> Box<dyn Node> {
        let mut n1 = self.parse_unary_expr();
        loop {
            let t = self.tok.peek();
            if( t.kind != TokenType::Id && t.kind != TokenType::Number && t.kind != TokenType::ParOpen) {
                break;
            };
            //don't consume the token yet...
            let op = Token {
                kind: TokenType::Mult,
                range: t.range.clone(),
                #[cfg(debug_assertions)]
                text: "implicit mult".to_string(),
            };
            let n2 = if t.kind == TokenType::ParOpen {
                Parser::reduce_list(self.parse_list_expr())
            } else {
                self.parse_postfix_expr()
            };
            let expr = BinExpr {
                node_data: NodeData::new(),
                expr1: n1,
                op,
                expr2: n2,
                implicit_mult: true,
            };
            n1 = Box::new(expr);
        };
        n1
    }

    fn parse_unary_expr(&mut self) -> Box<dyn Node> {
        let token = self.tok.peek();
        if token.kind == TokenType::Min {
            return Box::new( UnaryExpr {
                node_data: NodeData { unit: Unit::none(), has_errors: false},
                op: self.tok.next(),
                expr: self.parse_postfix_expr(),
            });
        }
        self.parse_postfix_expr()
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
                    postfix.postfix_id = Token { kind: TokenType::ClearUnit, range : Range { start: 0, end: 0, source_index: 0},
                        #[cfg(debug_assertions)]
                        text: "".to_string()
                    } //TODO: since range can be empty: use Option?
                }
                Box::new(postfix)
            },
            TokenType::Inc | TokenType::Dec => {
                let t = self.tok.next();
                let f_name = if t.kind == TokenType::Inc {
                    "inc"
                } else {
                    "dec"
                };
                self.create_call_for_operator(f_name, node, &t.range)
            },
            TokenType::Exclam => {
                let t = self.tok.next();
                self.create_call_for_operator("factorial", node, &t.range)
            },
            _ => node
        }
    }

    fn create_call_for_operator(&mut self, function_name: &str, arg:  Box<dyn Node>, range: &Range) -> Box<dyn Node> {
        Box::new(CallExpr {
            node_data: NodeData { unit: Unit::none(), has_errors: false},
            function_name: function_name.to_string(),
            function_name_range: range.clone(),
            arguments: Box::new( ListExpr {
                node_data: NodeData  { unit: Unit::none(), has_errors: false},
                nodes: vec![arg],
            }),
        })
    }

    // if an id is 'glued' to a primary expr, without a dot in between, it should be a unit.
    fn parse_unit_expr(&mut self) -> Box<dyn Node> {
        let mut expr = self.parse_primary_expr();
        match self.tok.peek().kind {
            TokenType::Id => {
                let id = self.tok.peek();
                let id_str = self.globals.get_text(&id.range).to_string();
                if self.code_block.scope.borrow().var_defs.contains(&id_str) {
                    return expr; //ignore this id - it's probably an implicit mult.
                }
                let id = self.tok.next();
                let it = expr.deref_mut();
                let nd = &mut it.get_node_data_mut();
                if nd.unit.is_empty() {
                    nd.unit.id = id_str;
                } else { //there's a 2nd unit glued to the expr as in: `(1m)mm`, so wrap the original expr in a UnitExpr.
                    return Box::new( UnitExpr {
                        node_data: NodeData { unit: Unit {id: id_str, range: Some(id.range.clone())}, has_errors: false},
                        node: expr,
                    })
                }
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

    fn parse_call_expr(&mut self, function_name: Token) -> Box<dyn Node> {
        let func_name_str = self.globals.get_text(&function_name.range);
        if TokenType::ParOpen != self.tok.peek().kind {
            let error = Error::build(ErrorId::FuncNoOpenPar, function_name.range.clone(), &[&func_name_str]);
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
                    let error = Error::build(ErrorId::Eos, function_name.range.clone(), &[&self.globals.get_text(&function_name.range)]);
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
        let error = Error::build(id, range, &[arg1]);
        self.errors.push(error);
    }

    fn parse_primary_expr(&mut self) -> Box<dyn Node> {
        match self.tok.peek().kind {
            TokenType::Number => self.parse_number_expr(),
            TokenType::Id => {
                let t = self.tok.next();
                let id = self.globals.get_text(&t.range);
                if self.code_block.scope.borrow().local_function_defs.contains_key(id) {
                    return self.parse_call_expr(t);
                } else {
                    if self.globals.global_function_defs.contains_key(id) {
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
                    let error = Error::build(ErrorId::Expected, range, &[")"]);
                    self.errors.push(error);
                }
                if expr.as_any().type_id() == TypeId::of::<BinExpr>() {
                    let bin_expr = expr.as_any_mut().downcast_mut::<BinExpr>().unwrap();
                    bin_expr.implicit_mult = false;
                }
                expr
            },
            TokenType::Pipe => {
                let t = self.tok.next().clone();
                self.parse_abs_operator(t)
            },
            _ => Box::new(NoneExpr { node_data: NodeData { unit: Unit::none(), has_errors: false,}, token: self.tok.next().clone()})
        }
    }

    fn parse_abs_operator(&mut self, token: Token) -> Box<dyn Node> {
        let expr = self.parse_add_expr();
        let list = Box::new( ListExpr {
            node_data: NodeData {unit: Unit::none(), has_errors: false},
            nodes: vec![expr],
        });
        let node = Box::new(CallExpr {
            node_data: NodeData {unit: Unit::none(), has_errors: false}, //TODO: create a default for node_data, AFTER units have been implemented fully.
            function_name: "abs".to_string(),
            function_name_range: token.range.clone(),
            arguments: list,
        });
        if !self.match_token(&TokenType::Pipe) {
            self.add_error(ErrorId::Expected, self.tok.peek().range.clone(), "|");
        }
        node
    }

    fn parse_number_expr(&mut self) -> Box<dyn Node> {
        //assuming type of token already checked.
        let token = self.tok.next();
        Box::new(ConstExpr { value: self.tok.get_number(), node_data: NodeData { unit: Unit::none(),has_errors: false,}, range: token.range.clone()})
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
