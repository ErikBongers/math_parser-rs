use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::rc::Rc;
use macros::{CastAny, Node};
use crate::errors::{Error, ErrorId};
use crate::resolver::globals::Globals;
use crate::resolver::scope::Scope;
use crate::resolver::unit::Unit;
use crate::tokenizer::cursor::{Number, Range};
use crate::tokenizer::Token;

#[derive(Clone)]
pub struct NodeData {
    pub unit: Unit,
    pub has_errors: bool
}

impl NodeData {
    pub fn new() -> Self {
        NodeData {
            unit: Unit::none(),
            has_errors: false,
        }
    }
}

pub trait HasRange{
    fn get_range(&self) -> Range;
}

pub trait Node: CastAny + HasRange {
    fn get_node_data(&self) -> &NodeData;
    fn get_node_data_mut(&mut self) -> &mut NodeData;

    fn is_implicit_mult(&self) -> bool {
        match self.as_any().type_id() {
            t if TypeId::of::<BinExpr>() == t => self.as_any().downcast_ref::<BinExpr>().unwrap().implicit_mult,
            _ => false
        }
    }
}
//emulating a base class like Partial: https://docs.rs/partially/latest/partially/

#[derive(CastAny, Node)]
pub struct NoneExpr {
    pub node_data: NodeData,
    pub token: Token, //may be EOT
}

impl HasRange for NoneExpr {
    fn get_range(&self) -> Range {
        self.token.range.clone()
    }
}
#[derive(CastAny, Node)]
pub struct UnitExpr {
    pub node_data: NodeData,
    pub node: Box<dyn Node>
}

impl HasRange for UnitExpr {
    fn get_range(&self) -> Range {
        self.node.get_range()
    }
}

#[derive(CastAny, Node)]
pub struct CommentExpr {
    pub node_data: NodeData,
    pub token: Token,
}

impl HasRange for CommentExpr {
    fn get_range(&self) -> Range {
        self.token.range.clone()
    }
}

#[derive(CastAny, Node)]
pub struct AssignExpr {
    pub node_data: NodeData,
    pub id: Token,
    pub expr: Box<dyn Node>,
}

impl HasRange for AssignExpr {
    fn get_range(&self) -> Range {
        &self.id.range + &self.expr.get_range()
    }
}

#[derive(CastAny, Node)]
pub struct BinExpr {
    pub node_data: NodeData,
    pub expr1: Box<dyn Node>,
    pub op: Token,
    pub expr2: Box<dyn Node>,
    pub implicit_mult: bool,
}
impl HasRange for BinExpr {
    fn get_range(&self) -> Range {
         &self.expr1.get_range() + &self.expr2.get_range()
    }
}

#[derive(CastAny, Node)]
pub struct UnaryExpr {
    pub node_data: NodeData,
    pub op: Token,
    pub expr: Box<dyn Node>,
}
impl HasRange for UnaryExpr {
    fn get_range(&self) -> Range {
         &self.expr.get_range() + &self.op.range
    }
}


pub enum ConstType { Numeric {number: Number}, FormattedString }
#[derive(CastAny, Node)]
pub struct ConstExpr {
    pub node_data: NodeData,
    pub const_type: ConstType,
    pub range: Range,
}

impl HasRange for ConstExpr {
    fn get_range(&self) -> Range {
        self.range.clone()
    }
}

#[derive(CastAny, Node)]
pub struct IdExpr {
    pub node_data: NodeData,
    pub id: Token,
}

impl HasRange for IdExpr {
    fn get_range(&self) -> Range {
        self.id.range.clone()
    }
}

#[derive(CastAny, Node)]
pub struct PostfixExpr {
    pub node_data: NodeData,
    pub node: Box<dyn Node>,
    pub postfix_id: Token,
}

impl HasRange for PostfixExpr {
    fn get_range(&self) -> Range {
        &self.node.get_range() + &self.postfix_id.range.clone()
    }
}

#[derive(CastAny, Node)]
pub struct Statement {
    pub node_data: NodeData,
    pub node: Box<dyn Node>,
}

impl HasRange for Statement {
    fn get_range(&self) -> Range {
        self.node.get_range()
    }
}

impl Statement {
    pub fn error(errors: &mut Vec<Error>, id: ErrorId, token: Token, arg1: &str) -> Statement {
        errors.push( Error::build(id, token.range.clone(), &[arg1]) );
        Statement {
            node: Box::new(NoneExpr { token, node_data: NodeData { has_errors: true, unit: Unit::none()}}),
            node_data: NodeData { has_errors: true, unit: Unit::none()}
        }
    }
}

#[derive(CastAny, Node)]
pub struct ListExpr {
    pub node_data: NodeData,
    pub nodes: Vec<Box<dyn Node>>,
}

impl HasRange for ListExpr {
    fn get_range(&self) -> Range {
        self.nodes
            .iter()
            .map(|node| node.get_range())
            .reduce(|r1, r2| &r1 + &r2).unwrap_or(Range::none())
    }
}

#[derive(CastAny, Node, Clone)]
pub struct FunctionDefExpr {
    pub node_data: NodeData,
    pub id: String, //Not a Token because id may be a decorated name in case of polymorphism.
    pub id_range: Range, //the undecorated functionname
    pub arg_names: Vec<String>,
}

impl HasRange for FunctionDefExpr {
    fn get_range(&self) -> Range {
        self.id_range.clone() //TODO: the function def should have the full range of the definition.
    }
}

#[derive(CastAny, Node)]
pub struct CallExpr {
    pub node_data: NodeData,
    pub function_name: String, //this may not be a stream range, but a translated function name: e.g. x++ -> _inc(x)
    pub function_name_range: Range,
    pub arguments: Box<dyn Node>
}

impl HasRange for CallExpr {
    fn get_range(&self) -> Range {
        let arg_range = self.arguments.get_range();
        if arg_range.is_none() {
            self.function_name_range.clone()
        } else {
            &self.function_name_range + &self.arguments.get_range()
        }
    }
}

#[derive(CastAny, Node)]
pub struct CodeBlock {
    pub node_data: NodeData,
    pub statements: Vec<Box<Statement>>,
    pub scope: Rc<RefCell<Scope>>,
}

impl CodeBlock {
    pub fn new(scope: RefCell<Scope>) -> Self {
        CodeBlock {
            node_data: NodeData::new(),
            scope: Rc::new(scope),
            statements: Vec::new(),
        }
    }
}

impl HasRange for CodeBlock {
    fn get_range(&self) -> Range {
        self.statements.iter()
            .map(|stmt| stmt.get_range())
            .reduce(|sum, range| &sum + &range).unwrap_or(Range::none()) //TODO: add the range of the start point of the block, which may be the beginning of the text or the opening '{'.
    }
}


#[derive(CastAny, Node)]
pub struct DefineExpr {
    pub node_data: NodeData,
    pub def_undef: Token,
    pub tokens: Vec<Token>,
}

impl HasRange for DefineExpr {
    fn get_range(&self) -> Range {
        &self.def_undef.range + &self.tokens.iter()
            .map(|token| &token.range)
            .fold(self.def_undef.range.clone(), |sum, range| &sum + range)
    }
}


pub fn print_nodes(expr: &Box<dyn Node>, indent: usize, globals: &Globals) {
    print!("{: <1$}", "", indent);
    let indent= indent+5;
    match expr.as_any().type_id() {
        t if TypeId::of::<ConstExpr>() == t => {
            let expr = expr.as_any().downcast_ref::<ConstExpr>().unwrap();
            let value_str = match &expr.const_type {
                ConstType::Numeric { number } => number.to_double().to_string(),
                ConstType::FormattedString => globals.get_text(&expr.range).to_string()
            };
            println!("{0}: {1}{2}", "ConstExpr", value_str, expr.node_data.unit.id);
        },
        t if TypeId::of::<BinExpr>() == t => {
            println!("{0}: {1:?}", "BinExpr", expr.as_any().downcast_ref::<BinExpr>().unwrap().op.kind);
            let bin_expr = expr.as_any().downcast_ref::<BinExpr>().unwrap();
            print_nodes(&bin_expr.expr1, indent, globals);
            print_nodes(&bin_expr.expr2, indent, globals);
        },
        t if TypeId::of::<NoneExpr>() == t => {
            println!("{0}", "NoneExpr");
        },
        t if TypeId::of::<ListExpr>() == t => {
            println!("{0}", "ListExpr");
            let list_expr = expr.as_any().downcast_ref::<ListExpr>().unwrap();
            for child in &list_expr.nodes {
                print_nodes(&child, indent, globals);
            }
        },
        t if TypeId::of::<AssignExpr>() == t => {
            println!("{0}", "AssignExpr");
            let assign_expr = expr.as_any().downcast_ref::<AssignExpr>().unwrap();
            print_nodes(&assign_expr.expr, indent, globals);
        },
        t if TypeId::of::<PostfixExpr>() == t => {
            println!("{0}", "PostfixExpr");
        },
        t if TypeId::of::<CommentExpr>() == t => {
            println!("{0}", "CommentExpr");
        },
        t if TypeId::of::<CallExpr>() == t => {
            println!("{0}", "CallExpr");
        },
        t if TypeId::of::<IdExpr>() == t => {
            println!("{0}", "IdExpr");
        },
        t if TypeId::of::<FunctionDefExpr>() == t => {
            println!("{0}", "FunctionDefExpr");
        },
        t if TypeId::of::<UnitExpr>() == t => {
            println!("{0}", "UnitExpr");
        },
        t if TypeId::of::<DefineExpr>() == t => {
            println!("{0}", "DefineExpr");
        },
        _ => {
            println!("{0}", "It's a dunno...");
        }
    }
}

