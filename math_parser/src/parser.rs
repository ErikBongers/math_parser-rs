use crate::errors;
use crate::errors::Error;
use crate::parser::nodes::{AssignExpr, BinExpr, CallExpr, CodeBlock, CommentExpr, ConstExpr, ConstType, Define, DefineExpr, DefineType, FunctionDefExpr, HasRange, IdExpr, ListExpr, Node, NodeType, NoneExpr, PostfixExpr, Statement, UnaryExpr, UnitExpr};
use crate::parser::nodes::DefineType::Precision;
use crate::globals::Globals;
use crate::tokenizer::cursor::Range;
use crate::tokenizer::peeking_tokenizer::PeekingTokenizer;
use crate::tokenizer::token_type::TokenType;
use crate::resolver::scope::Scope;
use crate::resolver::unit::Unit;
use crate::tokenizer::Token;
use crate::tokenizer::token_type::TokenType::{Div};

pub mod nodes;

pub struct Parser<'g, 'a, 't> {
    globals: &'g Globals,
    tok: &'a mut PeekingTokenizer<'t>,
    errors: &'a mut Vec<Error>,
    code_block: CodeBlock,
    mute_block: bool
}

impl<'g, 'a, 't> Into<CodeBlock> for Parser<'g, 'a, 't> {
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
            mute_block: false,
        }
    }

    pub fn parse(&mut self, for_block: bool, inherited_mute: bool) {
        self.mute_block = inherited_mute;
        while self.tok.peek().kind != TokenType::Eot {
            let stmt = self.parse_statement();
            self.code_block.statements.push(stmt);
            if self.tok.peek().kind == TokenType::CurlClose {
                if for_block {
                    return;
                } else {
                    self.errors.push(errors::unknown_expr("}", self.tok.peek().range.clone()));
                    self.tok.next();//avoid deadloop
                }
            }
        }
    }

    fn parse_statement(&mut self) -> Statement {
        let mut mute_line = false;
        if self.match_token(&TokenType::MuteLine) {
            mute_line = true;
        } else {
            if self.match_token(&TokenType::MuteStart) {
                self.mute_block = true;
            } else {
                if self.match_token(&TokenType::MuteEnd) {
                    self.mute_block = false;
                }
            }
        }
        if self.tok.peek().kind == TokenType::CurlOpen {
            let curl_open = self.tok.next();
            let block = self.parse_block(curl_open.range.clone());
            if !self.match_token(&TokenType::CurlClose) {
                self.errors.push(errors::expected("}", self.tok.peek().range.clone()));
            }
            return Statement { node: Node::boxed(NodeType::Block(block)), mute: mute_line | self.mute_block };
        }
        if let Some(stmt) = self.parse_echo_comment() {
            return stmt.set_mute(mute_line | self.mute_block);
        }
        if let Some(stmt) = self.parse_defines() {
            return stmt.set_mute(mute_line | self.mute_block);
        }
        if let Some(stmt) = self.parse_function_def() {
            return stmt.set_mute(mute_line | self.mute_block);
        }
        self.parse_expr_statement().set_mute(mute_line | self.mute_block)
    }

    fn parse_defines(&mut self) -> Option<Statement> {
        if self.tok.peek().kind == TokenType::Define || self.tok.peek().kind == TokenType::Undef {
            let t = self.tok.next();
            self.tok.set_nl_is_token(true);
            let mut defines: Vec<Define> = Vec::new();
            while self.tok.peek().kind != TokenType::Eot {
                if self.tok.peek().kind == TokenType::Newline || self.tok.peek().kind == TokenType::SemiColon {
                    self.tok.next();
                    break;
                }
                if let Some(define) = self.parse_define() { //parse_define always eats at least one token, so no risk of deadloop.
                    defines.push(define);
                }
            }
            self.tok.set_nl_is_token(false);
            return Some(Statement { mute: false, node: Node::boxed(NodeType::Define(DefineExpr {
                    def_undef: t,
                    defines,
                }))
            });
            }
        None
    }

    fn parse_define(&mut self) -> Option<Define>{
        let mut extra_range = None;
        let token = self.tok.next();
        let txt = self.globals.get_text(&token.range); //assuming TokenType::Id, not checking as the next match will cover it.
        let define_type = match txt {
            "ymd" => DefineType::Ymd,
            "dmy" => DefineType::Dmy,
            "mdy" => DefineType::Mdy,
            "precision" => {
                let eq = self.tok.peek();
                if eq.kind != TokenType::Eq {
                    self.errors.push(errors::expected("=", eq.range.clone()));
                    return None;
                }
                self.tok.next(); //eq
                let int = self.tok.peek();
                if int.kind != TokenType::Number {
                    self.errors.push(errors::expected("an integer", int.range.clone()));
                    return None;
                }
                let number_token = self.tok.next();
                extra_range = Some(number_token.range);
                let number = self.tok.get_number();
                Precision {number}
            },
            "date_units" => DefineType::DateUnits,
            "short_date_units" => DefineType::ShortDateUnits,
            "trig" => DefineType::Trig,
            "arithm" => DefineType::Arithm,
            "date" => DefineType::Date,
            "all" => DefineType::Default,
            "electric" => DefineType::Electric,
            "strict" => DefineType::Strict,
            "decimal_dot" | "dec_dot" | "dot" => DefineType::DecimalDot,
            "decimal_comma" | "dec_comma" | "comma" => DefineType::DecimalComma,
            "decimal_auto" | "dec_auto"  => DefineType::DecimalAuto,
            _ => {
                self.errors.push(errors::define_not_def(&txt, token.range.clone()));
                return None;
            }
        };
        let mut range = token.range.clone();
        if let Some(extra_range) = extra_range {
            range = &range + &extra_range;
        }
        Some(Define{ define_type, range})
    }

    fn parse_echo_comment(&mut self) -> Option<Statement> {
        let token = self.tok.peek();
        if token.kind != TokenType::EchoCommentLine {
            return None;
        };
        Some( Statement {
            node: Node::boxed(NodeType::Comment(CommentExpr { token: self.tok.next() })),
            mute: false,
        })
    }

    fn parse_function_def(&mut self) -> Option<Statement> {
        if self.tok.peek().kind != TokenType::Function {
            return None;
        };
        let start_range = self.tok.next().range;
        if self.tok.peek().kind != TokenType::Id {
            return Some(Statement::error(&mut self.errors, errors::expected_id(self.tok.peek().range.clone()), self.tok.peek().clone()));
        };
        let id = self.tok.next();

        if !self.match_token(&TokenType::ParOpen) {
            return Some(Statement::error(&mut self.errors, errors::expected("(", self.tok.peek().range.clone()), self.tok.peek().clone()));
        };

        let mut param_defs: Vec<String> = Vec::new();

        while self.tok.peek().kind == TokenType::Id {
            let txt = self.globals.get_text(&self.tok.next().range).to_string();
            param_defs.push(txt);
            if self.match_token(&TokenType::Comma) {
                continue;
            }
            if self.tok.peek().kind == TokenType::ParClose {
                break;
            }
            return Some(Statement::error(&mut self.errors, errors::expected(",` or `)", self.tok.peek().range.clone()), self.tok.peek().clone()));
        };

        if !self.match_token(&TokenType::ParClose) {
            return Some(Statement::error(&mut self.errors, errors::expected(")", self.tok.peek().range.clone()), self.tok.peek().clone()));
        };
        if self.tok.peek().kind != TokenType::CurlOpen {
            return Some(Statement::error(&mut self.errors, errors::expected("{", self.tok.peek().range.clone()), self.tok.peek().clone()));
        };
        let curl_open = self.tok.next();
        let new_code_block = self.parse_block(curl_open.range.clone());

        if self.tok.peek().kind != TokenType::CurlClose {
            return Some(Statement::error(&mut self.errors, errors::expected("}", self.tok.peek().range.clone()), self.tok.peek().clone()));
        };
        let token_end = self.tok.next();
        let fun_def_expr = FunctionDefExpr {
            id: self.globals.get_text(&id.range).to_string(),
            id_range: id.range.clone(),
            arg_names: param_defs,
            range: &start_range + &token_end.range,
        };
        self.code_block.scope.borrow_mut().add_local_function(new_code_block, &fun_def_expr);
        let node = Node::new(NodeType::FunctionDef(fun_def_expr));
        Some(Statement {
            node: Box::new(node),
            mute: false,
        })
    }

    fn parse_block(&mut self, block_start: Range) -> CodeBlock {
        let new_scope = Scope::copy_for_block(&self.code_block.scope);
        let new_code_block = CodeBlock::new(new_scope, block_start);
        let mut parser = Parser::new(&self.globals, &mut self.tok, &mut self.errors, new_code_block);
        parser.parse(true, self.mute_block);
        parser.into()
    }

    fn parse_expr_statement(&mut self) -> Statement {
        let mut stmt = Statement {
            node: self.parse_assign_expr(),
            mute: false,
        };
        match self.tok.peek().kind {
            TokenType::SemiColon => {
                self.tok.next();
            },
            TokenType::Eot => (),
            TokenType::CurlClose => (), //possible end of block
            _ => {
                let t = self.tok.next(); //avoid dead loop!
                self.errors.push(errors::expected(";", t.range.clone()));
                stmt.node.has_errors = true;
            }
        };
        stmt
    }

    fn parse_assign_expr(&mut self) -> Box<Node> {
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
            let eq = self.tok.next();
            let assign_expr = AssignExpr {
                id,
                expr: Parser::reduce_list(Node::boxed(NodeType::List(self.parse_list_expr())))
            };

            if let NodeType::None(none_expr) = &assign_expr.expr.expr {
                if none_expr.token.kind == TokenType::Eot {
                    self.errors.push(errors::eos(Range { start: eq.range.end, ..eq.range }));
                }
            }

            let txt = self.globals.get_text(&assign_expr.id.range).to_string();
            self.code_block.scope.borrow_mut().var_defs.insert(txt);
            return Node::boxed(NodeType::Assign(assign_expr));
        }
        //build this expression: AssignExpr {id, BinExpr{ IdExpr, op, expr }}
        let eq_op = self.tok.next();
        let id_expr = IdExpr {
            id: id.clone(),
        };

        let expr : Box<Node> = match op_type {
            EqPlus | EqMin | EqMult | EqDiv => {
                let bin_op = match op_type {
                    EqPlus => Plus,
                    EqMin => Min,
                    EqMult => Mult,
                    EqDiv => Div,
                    _ => unreachable!()
                };
                Node::boxed(NodeType::Binary(BinExpr {
                    expr1: Node::boxed(NodeType::Id(id_expr)),
                    op: Token {
                        kind: bin_op,
                        range: eq_op.range,
                        #[cfg(debug_assertions)]
                        text: "Eq_xxx".to_string(),
                    } ,
                    expr2: self.parse_add_expr(),
                    implicit_mult: false,
                }))
            },
            EqUnit => {
                let id_token = if self.tok.peek().kind == TokenType::Id { //assume id is a variable with a unit we'd like to apply.
                    self.tok.next()
                } else {
                    Token {
                        kind: TokenType::ClearUnit,
                        range: Range::none(self.tok.source_index()),
                        #[cfg(debug_assertions)]
                        text: "".to_string(),
                    }
                };
                Node::boxed(NodeType::Postfix(PostfixExpr {
                    node: Node::boxed(NodeType::Id(id_expr)),
                    postfix_id: id_token,
                }))
            },
            _ => unreachable!("expected a Eq operator.")
        };

        let assign_expr = AssignExpr {
            id,
            expr,
        };
        Node::boxed(NodeType::Assign(assign_expr))
    }

    fn parse_add_expr(&mut self) -> Box<Node> {
        let mut expr1 = self.parse_mult_expr();
        loop {
            match self.tok.peek().kind {
                TokenType::Plus | TokenType::Min => {
                    let op = self.tok.next().clone();
                    let expr2 = self.parse_mult_expr();
                    expr1 = Node::boxed(NodeType::Binary(BinExpr { expr1, op, expr2, implicit_mult: false }))
                }
                _ => break
            }
        };
        expr1
    }

    fn parse_mult_expr(&mut self) -> Box<Node> {
        let mut expr1 = self.parse_power_expr();
        loop {
            match self.tok.peek().kind {
                TokenType::Mult | TokenType::Div | TokenType::Percent | TokenType::Modulo => {
                    let op = self.tok.next().clone();
                    let expr2 = self.parse_power_expr();
                    if op.kind == Div {
                        if expr2.expr.is_implicit_mult() {
                            self.errors.push(errors::w_div_impl_mult(expr2.get_range().clone()));
                        }
                    }
                    expr1 = Node::boxed(NodeType::Binary(BinExpr { expr1, op, expr2, implicit_mult: false }))
                }
                _ => break
            }
        };
        expr1
    }

    fn parse_power_expr(&mut self) -> Box<Node> {
        let mut expr1 = self.parse_implicit_mult();
        loop {
            match self.tok.peek().kind {
                TokenType::Power => {
                    let op = self.tok.next().clone();
                    let expr2 = self.parse_power_expr(); //right associative!
                    let bin_expr = BinExpr { expr1, op, expr2, implicit_mult: false };
                    if bin_expr.expr1.expr.is_implicit_mult() || bin_expr.expr2.expr.is_implicit_mult() {
                        self.errors.push(errors::w_pow_impl_mult(bin_expr.get_range().clone()));
                    }
                    expr1 = Node::boxed(NodeType::Binary(bin_expr));

                }
                _ => break
            }
        }
        expr1
    }

    fn parse_implicit_mult(&mut self) -> Box<Node> {
        let mut n1 = self.parse_unary_expr();
        loop {
            let t = self.tok.peek();
            if t.kind != TokenType::Id && t.kind != TokenType::Number && t.kind != TokenType::ParOpen {
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
                Parser::reduce_list(Node::boxed(NodeType::List(self.parse_list_expr())))
            } else {
                self.parse_postfix_expr()
            };
            let expr = BinExpr {
                expr1: n1,
                op,
                expr2: n2,
                implicit_mult: true,
            };
            n1 = Node::boxed(NodeType::Binary(expr));
        };
        n1
    }

    fn parse_unary_expr(&mut self) -> Box<Node> {
        let token = self.tok.peek();
        if token.kind == TokenType::Min {
            return Node::boxed( NodeType::Unary(UnaryExpr {
                op: self.tok.next(),
                expr: self.parse_postfix_expr(),
            }));
        }
        self.parse_postfix_expr()
    }

    fn parse_postfix_expr(&mut self) -> Box<Node> {
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

    fn parse_one_postfix(&mut self, node: Box<Node>) -> Box<Node> {
        match self.tok.peek().kind {
            TokenType::Dot => {
                let dot = self.tok.next();
                let t = self.tok.peek();
                let t_type = &t.kind.clone();
                let mut postfix = PostfixExpr { postfix_id: t.clone(), node};
                if t_type == &TokenType::Id {
                    self.tok.next();
                    // postfix.postfix_id already set!
                } else {
                    postfix.postfix_id = Token { kind: TokenType::ClearUnit, range : Range { end: dot.range.end, ..dot.range}, //range: zero length, right behind dot.
                        #[cfg(debug_assertions)]
                        text: "".to_string()
                    }
                }
                Node::boxed(NodeType::Postfix(postfix))
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

    fn create_call_for_operator(&mut self, function_name: &str, arg:  Box<Node>, range: &Range) -> Box<Node> {
        Node::boxed(NodeType::Call( CallExpr {
            function_name: function_name.to_string(),
            function_name_range: range.clone(),
            arguments: vec![arg],
            par_close_range: range.clone(), //just use the same range, there's no real arguments here.
        }))
    }

    // if an id is 'glued' to a primary expr, without a dot in between, it should be a unit.
    fn parse_unit_expr(&mut self) -> Box<Node> {
        let mut expr = self.parse_primary_expr();
        if let TokenType::Id = self.tok.peek().kind {
            let id = self.tok.peek();
            let id_str = self.globals.get_text(&id.range).to_string();
            if self.code_block.scope.borrow().var_defs.contains(&id_str) {
                return expr; //ignore this id - it's probably an implicit mult.
            }
            let id = self.tok.next();
            if expr.unit.is_empty() {
                expr.unit = Unit::from_id(&id_str, Some(id.range.clone()));
            } else { //there's a 2nd unit glued to the expr as in: `(1m)mm`, so wrap the original expr in a UnitExpr.
                let mut unit_node = Node::new(NodeType::Unit(UnitExpr {
                    node: expr,
                    range: id.range.clone(),
                }));
                unit_node.unit = Unit::from_id(&id_str, Some(id.range.clone()));
                return Box::new(unit_node);
            }
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

    fn parse_call_expr(&mut self, function_name: Token) -> Box<Node> {
        let func_name_str = self.globals.get_text(&function_name.range);
        if TokenType::ParOpen != self.tok.peek().kind {
            let error = errors::func_no_open_par(&func_name_str, function_name.range.clone());
            self.errors.push(error);
            let mut node = Node::new(NodeType::None(NoneExpr{token: function_name.clone()}));
            node.has_errors = true;
            return Box::new(node)
        }
        self.tok.next();// eat `(`
        let mut list_expr = self.parse_list_expr();
        //first argument may be NONE, with a token EOT, which is an invalid argument list in this case.
        if list_expr.nodes.len() == 1 {
            if let NodeType::None(none_expr) = &list_expr.nodes.first().unwrap().expr { //unwrap: len == 1
                if none_expr.token.kind == TokenType::Eot {
                    let error =errors::eos(function_name.range.clone());
                    self.errors.push(error);
                    let mut node =  Node::new(NodeType::None(NoneExpr{token: function_name}));
                    node.has_errors = true;
                    return Box::new(node);
                } else {
                    list_expr.nodes.clear();
                }
            }
        }
        if self.tok.peek().kind != TokenType::ParClose {
            self.errors.push(errors::expected(")", self.tok.peek().range.clone()));
        }
        let par_close = self.tok.next(); // ')'
        Node::boxed(NodeType::Call(CallExpr {
            function_name: func_name_str.to_string(),
            function_name_range: function_name.range.clone(),
            arguments: list_expr.nodes,
            par_close_range: par_close.range
        }))
    }

    fn parse_primary_expr(&mut self) -> Box<Node> {
        match self.tok.peek().kind {
            TokenType::Number => self.parse_number_expr(),
            TokenType::Id => {
                let t = self.tok.next();
                let id = self.globals.get_text(&t.range);
                if self.code_block.scope.borrow().function_exists(id, self.globals) {
                    return self.parse_call_expr(t);
                }
                Node::boxed(NodeType::Id(IdExpr {
                    id: t,
                }))
            }
            TokenType::ParOpen => {
                self.tok.next();
                let mut expr = Parser::reduce_list(Node::boxed(NodeType::List(self.parse_list_expr())));
                if !self.match_token(&TokenType::ParClose) {
                    let error = errors::expected(")", expr.get_range());
                    self.errors.push(error);
                }
                if let NodeType::Binary(ref mut bin_expr) =  &mut expr.expr {
                    bin_expr.implicit_mult = false;
                }
                expr
            },
            TokenType::Pipe => {
                let t = self.tok.next().clone();
                self.parse_abs_operator(t)
            },
            TokenType::QuotedStr => {
                let t = self.tok.next();
                Node::boxed(NodeType::Const(ConstExpr {
                    const_type: ConstType::FormattedString,
                    range: t.range,
                }))
            }
            // if nothing meaningfull found, don't report an error yet as this will be too generic : "Unexpected..."
            _ => Node::boxed(NodeType::None(NoneExpr { token: self.tok.peek().clone()}))
        }
    }

    fn parse_abs_operator(&mut self, token: Token) -> Box<Node> {
        let expr = self.parse_add_expr();
        if self.tok.peek().kind != TokenType::Pipe {
            self.errors.push(errors::expected("|", self.tok.peek().range.clone()));
        }
        let node = Node::boxed(NodeType::Call(CallExpr {
            function_name: "abs".to_string(),
            function_name_range: token.range.clone(),
            arguments: vec![expr],
            par_close_range: self.tok.next().range, // closing pipe.
        }));
        node
    }

    fn parse_number_expr(&mut self) -> Box<Node> {
        //assuming type of token already checked.
        let token = self.tok.next();
        Node::boxed(NodeType::Const(ConstExpr { const_type: ConstType::Numeric { number: self.tok.get_number()}, range: token.range.clone()}))
    }

    fn reduce_list(mut node: Box<Node>) -> Box<Node> {
        if let NodeType::List(ref mut list_expr) = &mut node.expr {
            return if list_expr.nodes.len() == 1 {
                list_expr.nodes.remove(0)
            } else {
                node
            }
        }
        node
    }

    fn parse_list_expr(&mut self) -> ListExpr {
        let mut list_expr = ListExpr { nodes: Vec::new()};
        loop {
            let expr = self.parse_add_expr();
            list_expr.nodes.push(expr);
            if let NodeType::None(_) = list_expr.nodes.last().unwrap().expr { //unwrap: push() guarantees there's a last()
                break;
            }
            if self.tok.peek().kind != TokenType::Comma {
                break;
            }
            self.tok.next();
        }
        list_expr
    }
}
