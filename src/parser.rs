use crate::parser::nodes::{BinExpr, ConstExpr, Node, NodeData, NoneExpr, Statement};
use crate::tokenizer::cursor::Range;
use crate::tokenizer::peeking_tokenizer::PeekingTokenizer;
use crate::tokenizer::token_type::TokenType;

pub mod nodes;

struct CodeBlock {
    statements: Vec<Statement>
}

struct Parser<'a> {
    tok: &'a mut PeekingTokenizer<'a>,
    statement_start: Range,
    code_block: &'a CodeBlock,
}

impl<'a> Parser<'a> {
    fn parse(&mut self) {
        while self.tok.peek().kind != TokenType::Eot {
            let stmt = self.parse_statement();
        }
    }

    fn parse_statement(&mut self) -> Statement {
        Statement {
            node: self.parse_add_expr(),
            node_data: NodeData {
                error: 0,
                unit: 0
            }
        }
    }

    fn parse_add_expr(&mut self) -> Box<dyn Node> {
        let expr1 = self.parse_mult_expr();
        match self.tok.peek().kind {
            TokenType::Plus | TokenType::Min => {
                let op = self.tok.next().clone();
                let expr2 = self.parse_mult_expr();
                Box::new(BinExpr { expr1, op, expr2, node_data: NodeData { error: 0, unit: 0 } })
            },
            _ => Box::new(NoneExpr{ node_data: NodeData{ error: 0, unit: 0}})
        }
    }

    fn parse_mult_expr(&mut self) -> Box<dyn Node> {
        let expr1 = self.parse_primary_expr();
        match self.tok.peek().kind {
            TokenType::Mult | TokenType::Div | TokenType::Percent | TokenType::Modulo => {
                let op = self.tok.next().clone();
                let expr2 = self.parse_primary_expr();
                Box::new(BinExpr { expr1, op, expr2, node_data: NodeData { error: 0, unit: 0 } })
            },
            _ => Box::new(NoneExpr{ node_data: NodeData{ error: 0, unit: 0}})
        }
    }

    fn parse_primary_expr(&mut self) -> Box<dyn Node> {
        match self.tok.peek().kind {
            TokenType::Number => self.parse_number_expr(),
            _ => Box::new(NoneExpr { node_data: NodeData { unit: 0, error: 0}})
        }
    }

    fn parse_number_expr(&mut self) -> Box<dyn Node> {
        //assuming type of token already checked.
        self.tok.next();
        Box::new(ConstExpr { value: self.tok.get_number(), node_data: NodeData { error: 0, unit: 0}})
    }

}

