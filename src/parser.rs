use std::any::{Any, TypeId};
use cast_any::CastAny;
use crate::parser::nodes::{BinExpr, ConstExpr, Node, NodeData, NoneExpr, Statement};
use crate::tokenizer::cursor::Range;
use crate::tokenizer::peeking_tokenizer::PeekingTokenizer;
use crate::tokenizer::token_type::TokenType;

pub mod nodes;

pub(crate) struct CodeBlock {
    pub(crate) statements: Vec<Statement>
}

pub(crate) struct Parser<'a> {
    tok: &'a mut PeekingTokenizer<'a>,
    statement_start: Range,
    pub(crate) code_block: &'a mut CodeBlock,
}

impl<'a> Parser<'a> {
    pub(crate) fn new (tok: &'a mut PeekingTokenizer<'a>, code_block: &'a mut CodeBlock) -> Self {
        Parser {
            tok,
            code_block,
            statement_start: Range { start: 0, end: 0, source_index: 0}
        }
    }

    pub(crate) fn parse(&mut self) {
        while self.tok.peek().kind != TokenType::Eot {
            let stmt = self.parse_statement();
            self.code_block.statements.push(stmt);
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
            _ => expr1
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
            _ => expr1
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

pub fn print_nodes(expr: &Box<dyn Node>, indent: usize) {
    print!("{: <1$}", "", indent);
    let indent= indent+5;
    match expr.as_any().type_id() {
        t if TypeId::of::<ConstExpr>() == t => {
            let expr = expr.as_any().downcast_ref::<ConstExpr>().unwrap();
            println!("{0}: {1}", "ConstExpr", expr.as_any().downcast_ref::<ConstExpr>().unwrap().value.significand);
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
        _ => {
            println!("{0}", "It's a dunno...");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{CodeBlock, Parser};
    use crate::parser::nodes::{BinExpr, ConstExpr};
    use crate::tokenizer::peeking_tokenizer::PeekingTokenizer;
    use crate::tokenizer::token_type::TokenType;

    #[test]
    fn test_math_expression() {
        let txt = "2 + 3 * 4";
        let mut tok = PeekingTokenizer::new(txt);
        let mut code_block = CodeBlock { statements: Vec::new()};
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
