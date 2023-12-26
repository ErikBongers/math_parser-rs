use std::any::{Any, TypeId};
use std::ops::DerefMut;
use macros::CastAny;
use crate::errors::Error;
use crate::parser::nodes::{BinExpr, ConstExpr, Node, NodeData, NoneExpr, PostfixExpr, Statement};
use crate::tokenizer::cursor::Range;
use crate::tokenizer::peeking_tokenizer::PeekingTokenizer;
use crate::tokenizer::token_type::TokenType;
use crate::resolver::scope::Scope;
use crate::resolver::unit::Unit;
use crate::tokenizer::Token;

pub mod nodes;

pub struct CodeBlock<'a> {
    pub statements: Vec<Statement>,
    pub scope: &'a Scope<'a>,
    pub errors: Vec<Error>,
}

impl<'a> CodeBlock<'a> {
    pub fn new(scope: &'a Scope<'_>) -> Self {
        CodeBlock {
            scope,
            statements: Vec::new(),
            errors: Vec::new(),
        }
    }
}

pub struct Parser<'a> {
    tok: &'a mut PeekingTokenizer<'a>,
    statement_start: Range,
    pub code_block: &'a mut CodeBlock<'a>,
}

impl<'a> Parser<'a> {
    pub fn new (tok: &'a mut PeekingTokenizer<'a>, code_block: &'a mut CodeBlock<'a>) -> Self {
        Parser {
            tok,
            code_block,
            statement_start: Range { start: 0, end: 0, source_index: 0}
        }
    }

    pub fn parse(&mut self) {
        while self.tok.peek().kind != TokenType::Eot {
            let stmt = self.parse_statement();
            self.code_block.statements.push(stmt);
        }
    }

    fn parse_statement(&mut self) -> Statement {
        Statement {
            node: self.parse_add_expr(),
            node_data: NodeData {
                unit: Unit::none(),
                has_errors: false,
            }
        }
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
                    postfix.postfix_id = Token { kind: TokenType::Nullptr, range : Range { start: 0, end: 0, source_index: 0}} //TODO: since range can be empty: use Option?
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
                let id= self.code_block.scope.globals.get_text(&id.range).to_string();
                let it = expr.deref_mut();
                let mut nd = &mut it.get_node_data();
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

    fn parse_primary_expr(&mut self) -> Box<dyn Node> {
        match self.tok.peek().kind {
            TokenType::Number => self.parse_number_expr(),
            TokenType::ParOpen => {
                self.tok.next();
                let mut expr = self.parse_add_expr();
                if !self.match_token(&TokenType::ParClose) {
                    //TODO: report error.
                }
                if expr.as_any().type_id() == TypeId::of::<BinExpr>() {
                    let mut bin_expr = expr.as_any_mut().downcast_mut::<BinExpr>().unwrap();
                    bin_expr.implicit_mult = false;
                }
                expr
            },
            _ => Box::new(NoneExpr { node_data: NodeData { unit: Unit::none(), has_errors: false,}})
        }
    }

    fn parse_number_expr(&mut self) -> Box<dyn Node> {
        //assuming type of token already checked.
        self.tok.next();
        Box::new(ConstExpr { value: self.tok.get_number(), node_data: NodeData { unit: Unit::none(),has_errors: false,}})
    }

}

pub fn print_nodes(expr: &Box<dyn Node>, indent: usize) {
    print!("{: <1$}", "", indent);
    let indent= indent+5;
    match expr.as_any().type_id() {
        t if TypeId::of::<ConstExpr>() == t => {
            let expr = expr.as_any().downcast_ref::<ConstExpr>().unwrap();
            println!("{0}: {1}{2}", "ConstExpr", expr.as_any().downcast_ref::<ConstExpr>().unwrap().value.significand, expr.node_data.unit.id);
        },
        t if TypeId::of::<BinExpr>() == t => {
            println!("{0}: {1:?}", "BinExpr", expr.as_any().downcast_ref::<BinExpr>().unwrap().op.kind);
            let bin_expr = expr.as_any().downcast_ref::<BinExpr>().unwrap();
            print_nodes(&bin_expr.expr1, indent);
            print_nodes(&bin_expr.expr2, indent);
        },
        t if TypeId::of::<NoneExpr>() == t => {
            println!("{0}", "NoneExpr");
        },
        t if TypeId::of::<PostfixExpr>() == t => {
            println!("{0}", "PostfixExpr");
        },
        _ => {
            println!("{0}", "It's a dunno...");
        }
    }
}

#[cfg(test)]
mod tests {
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
        let scope = Scope::new(&mut globals);
        let mut code_block = CodeBlock::new(&scope);
        let mut parser = Parser::new(&mut tok, &mut code_block);
        parser.parse();
        let stmt = parser.code_block.statements.first().expect("There should be a statement here.");
        let root = stmt.node.as_any().downcast_ref::<BinExpr>().expect("There should be a BinExpr here.");
        let expr1 = root.expr1.as_any().downcast_ref::<ConstExpr>().expect("There should be a ConstExpr here.");
        assert_eq!(root.op.kind, TokenType::Plus);
        let expr2 = root.expr2.as_any().downcast_ref::<BinExpr>().expect("There should be a BinExpr here.");
        let expr2_1 = expr2.expr1.as_any().downcast_ref::<ConstExpr>().expect("There should be a ConstExpr here.");
        assert_eq!(expr2.op.kind, TokenType::Mult);
        let expr2_2 = expr2.expr2.as_any().downcast_ref::<ConstExpr>().expect("There should be a ConstExpr here.");
    }
}
