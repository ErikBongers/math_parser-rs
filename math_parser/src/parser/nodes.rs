use std::any::{Any, TypeId};
use macros::{CastAny, Node};
use crate::resolver::unit::Unit;
use crate::tokenizer::cursor::{Number, Range};
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
    pub token: Token, //may be EOT
}

#[derive(CastAny, Node)]
pub struct AssignExpr {
    pub node_data: NodeData,
    pub id: Token,
    pub expr: Box<dyn Node>,
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
pub struct IdExpr {
    pub node_data: NodeData,
    pub id: Token,
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

#[derive(CastAny, Node)]
pub struct ListExpr {
    pub node_data: NodeData,
    pub nodes: Vec<Box<dyn Node>>,
}

#[derive(CastAny, Node)]
pub struct FunctionDefExpr {
    pub node_data: NodeData,
    pub id: String, //Not a Token because id may be a decorated name in case of polymorphism.
    pub arg_names: Vec<String>,
}

#[derive(CastAny, Node)]
pub struct CallExpr {
    pub node_data: NodeData,
    pub function_name: String, //this may not be a stream range, but a translated function name: e.g. x++ -> _inc(x)
    pub function_name_range: Range,
    pub arguments: Box<dyn Node>
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
        t if TypeId::of::<ListExpr>() == t => {
            println!("{0}", "ListExpr");
            let list_expr = expr.as_any().downcast_ref::<ListExpr>().unwrap();
            for child in &list_expr.nodes {
                print_nodes(&child, indent);
            }
        },
        t if TypeId::of::<AssignExpr>() == t => {
            println!("{0}", "AssignExpr");
            let assign_expr = expr.as_any().downcast_ref::<AssignExpr>().unwrap();
            print_nodes(&assign_expr.expr, indent);
        },
        t if TypeId::of::<PostfixExpr>() == t => {
            println!("{0}", "PostfixExpr");
        },
        t if TypeId::of::<CallExpr>() == t => {
            println!("{0}", "CallExpr");
        },
        t if TypeId::of::<IdExpr>() == t => {
            println!("{0}", "IdExpr");
        },
        _ => {
            println!("{0}", "It's a dunno...");
        }
    }
}

