use std::any::Any;
use macros::{CastAny, Node};
use crate::resolver::unit::Unit;
use crate::tokenizer::cursor::Number;
use crate::tokenizer::Token;

pub struct NodeData {
    pub unit: Unit,
    pub has_errors: bool
}
pub trait Node: CastAny {
    fn get_node_data(&mut self) -> &mut NodeData;
}
//emulating a base class like Partial: https://docs.rs/partially/latest/partially/

#[derive(CastAny, Node)]
pub struct NoneExpr {
    pub node_data: NodeData,
}

#[derive(CastAny, Node)]
pub struct BinExpr {
    pub node_data: NodeData,
    pub expr1: Box<dyn Node>,
    pub op: Token,
    pub expr2: Box<dyn Node>,
    pub implicit_mult: bool,
}

#[derive(CastAny, Node)]
pub struct ConstExpr {
    pub node_data: NodeData,
    pub value: Number,
}

#[derive(CastAny, Node)]
pub struct PostfixExpr {
    pub node_data: NodeData,
    pub node: Box<dyn Node>,
    pub postfix_id: Token,
}

#[derive(CastAny, Node)]
pub struct Statement {
    pub node_data: NodeData,
    pub node: Box<dyn Node>,
    //TODO: if statement contains a codeBlock: should that just be a Node? This would allow for a codeBlock to return a last value as it's own value.
}

