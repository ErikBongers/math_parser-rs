use std::cell::RefCell;
use std::rc::Rc;
use crate::errors::Error;
use crate::globals::Globals;
use crate::number::Number;
use crate::resolver::scope::Scope;
use crate::resolver::unit::Unit;
use crate::tokenizer::cursor::Range;
use crate::tokenizer::Token;

pub trait HasRange{
    fn get_range(&self) -> Range;
}

pub struct Node {
    pub unit: Unit,
    pub has_errors: bool,
    pub expr: NodeType,
}

impl Node {
    pub fn new (expr: NodeType) -> Node {
        Node {
            unit: Unit::none(),
            has_errors: false,
            expr,
        }
    }

    pub fn boxed(expr: NodeType) -> Box<Node> {
       Box::new(Node::new(expr))
    }
}

impl HasRange for Node {
    fn get_range(&self) -> Range {
        self.expr.get_range()
    }
}

pub enum NodeType {
    None(NoneExpr),
    Unit(UnitExpr),
    Comment(CommentExpr),
    Assign(AssignExpr),
    Binary(BinExpr),
    Unary(UnaryExpr),
    Const(ConstExpr),
    Id(IdExpr),
    Postfix(PostfixExpr),
    List(ListExpr),
    FunctionDef(FunctionDefExpr),
    Call(CallExpr),
    Block(CodeBlock),
    Define(DefineExpr),
    Pragma(PragmaExpr),
    Assignable(AssignableExpr)
}

impl NodeType {
    pub fn is_implicit_mult(&self) -> bool {
        if let Binary(bin_expr)  = self{
            bin_expr.implicit_mult
        } else {
            false
        }
    }
}

use NodeType as N;
use crate::parser::nodes::N::Binary;

impl HasRange for NodeType {
    fn get_range(&self) -> Range {
        match self {
            N::None(expr) => expr.get_range(),
            N::Unit(expr) => expr.get_range(),
            N::Comment(expr) => expr.get_range(),
            N::Assign(expr) => expr.get_range(),
            N::Binary(expr) => expr.get_range(),
            N::Unary(expr) => expr.get_range(),
            N::Const(expr) => expr.get_range(),
            N::Id(expr) => expr.get_range(),
            N::Postfix(expr) => expr.get_range(),
            N::List(expr) => expr.get_range(),
            N::FunctionDef(expr) => expr.get_range(),
            N::Call(expr) => expr.get_range(),
            N::Block(expr) => expr.get_range(),
            N::Define(expr) => expr.get_range(),
            N::Pragma(expr) => expr.get_range(),
            N::Assignable(expr) => expr.get_range(),
        }
    }
}

pub struct NoneExpr {
    pub token: Token, //may be EOT
}

impl HasRange for NoneExpr {
    fn get_range(&self) -> Range {
        self.token.range.clone()
    }
}

pub struct UnitExpr {
    pub node: Box<Node>,
}

impl HasRange for UnitExpr {
    fn get_range(&self) -> Range {
        self.node.get_range()
    }
}

#[derive(Clone)]
pub struct CommentExpr {
    pub token: Token,
}

impl HasRange for CommentExpr {
    fn get_range(&self) -> Range {
        self.token.range.clone()
    }
}

pub struct AssignableExpr {
    pub id: Token,
    pub fragment: Option<Token>,
}

impl HasRange for AssignableExpr {
    fn get_range(&self) -> Range {
        let mut range = self.id.range.clone();
        if let Some(fragment)  = &self.fragment {
            range = &range + &fragment.range;
        }
        range
    }
}

pub struct AssignExpr {
    pub assignable: AssignableExpr,
    pub expr: Box<Node>,
}

impl HasRange for AssignExpr {
    fn get_range(&self) -> Range {
        &self.assignable.get_range() + &self.expr.get_range()
    }
}

pub struct BinExpr {
    pub expr1: Box<Node>,
    pub op: Token,
    pub expr2: Box<Node>,
    pub implicit_mult: bool,
}

impl HasRange for BinExpr {
    fn get_range(&self) -> Range {
         &self.expr1.get_range() + &self.expr2.get_range()
    }
}

pub struct UnaryExpr {
    pub op: Token,
    pub expr: Box<Node>,
}

impl HasRange for UnaryExpr {
    fn get_range(&self) -> Range {
         &self.expr.get_range() + &self.op.range
    }
}

pub enum ConstType { Numeric {number: Number}, FormattedString }

pub struct ConstExpr {
    pub const_type: ConstType,
    pub range: Range,
}

impl HasRange for ConstExpr {
    fn get_range(&self) -> Range {
        self.range.clone()
    }
}

pub struct IdExpr {
    pub id: Token,
}

impl HasRange for IdExpr {
    fn get_range(&self) -> Range {
        self.id.range.clone()
    }
}

pub struct PostfixExpr {
    pub node: Box<Node>,
    pub postfix_id: Token,
}

impl HasRange for PostfixExpr {
    fn get_range(&self) -> Range {
        if self.postfix_id.range.is_none() {
            self.node.get_range()
        } else {
            &self.node.get_range() + &self.postfix_id.range.clone()
        }
    }
}

pub struct Statement {
    pub node: Box<Node>,
    pub mute: bool
}

impl HasRange for Statement {
    fn get_range(&self) -> Range {
        self.node.get_range()
    }
}

impl Statement {
    pub fn error(errors: &mut Vec<Error>, error: Error, token: Token) -> Statement {
        errors.push( error);
        Statement {
            node: Box::new( Node { expr: N::None(NoneExpr { token }), unit: Unit::none(), has_errors: true }),
            mute: false
        }
    }

    pub fn set_mute(mut self, mute: bool) -> Self {
        self.mute = mute;
        self
    }
}

pub struct ListExpr {
    pub nodes: Vec<Box<Node>>,
}

impl HasRange for ListExpr {
    fn get_range(&self) -> Range {
        self.nodes
            .iter()
            .map(|node| node.get_range())
            .reduce(|r1, r2| &r1 + &r2).expect("No range found. List may not be empty.")
    }
}

#[derive(Clone)]
pub struct FunctionDefExpr {
    pub id: String, //Not a Token because id may be a decorated name in case of polymorphism.
    pub id_range: Range, //the undecorated functionname
    pub arg_names: Vec<String>,
    pub range: Range,
    //don't add CodeBlock as it can't moved out of the (immutable) AST and it can't be referenced from the AST without lifetime issues
}

impl HasRange for FunctionDefExpr {
    fn get_range(&self) -> Range {
        self.range.clone()
    }
}

pub struct CallExpr {
    pub function_name: String, //this may not be a stream range, but a translated function name: e.g. x++ -> _inc(x)
    pub function_name_range: Range,
    pub arguments: Vec<Box<Node>>,
    pub par_close_range: Range,
}

impl HasRange for CallExpr {
    fn get_range(&self) -> Range {
        &self.function_name_range + &self.par_close_range
    }
}

pub struct CodeBlock {
    pub block_start: Range,
    pub statements: Vec<Statement>,
    pub scope: Rc<RefCell<Scope>>,
    errors: Vec<Error>,
}

impl CodeBlock {
    pub fn new(scope: RefCell<Scope>, block_start: Range) -> Self {
        CodeBlock {
            block_start,
            scope: Rc::new(scope),
            statements: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn get_parser_errors(&self) -> &Vec<Error> {
        &self.errors
    }

    pub fn append_errors(&mut self, errors: &Vec<Error>) {
        self.errors.append(&mut errors.clone());
    }
}

impl HasRange for CodeBlock {
    fn get_range(&self) -> Range {
        &self.block_start +  &self.statements.iter()
            .map(|stmt| stmt.get_range())
            .reduce(|sum, range| &sum + &range).unwrap_or(self.block_start.clone())
    }
}

pub enum DefineType {
    Dmy,
    Ymd,
    Mdy,
    Precision { number: Number },
    DateUnits,
    ShortDateUnits,
    Trig,
    Arithm,
    Date,
    Default,
    Electric,
    Strict,
    DecimalDot,
    DecimalComma,
    DecimalAuto,
}

pub struct Define {
    pub define_type: DefineType,
    pub range:Range,
}

pub struct DefineExpr {
    pub def_undef: Token,
    pub defines: Vec<Define>,
}

impl HasRange for DefineExpr {
    fn get_range(&self) -> Range {
        &self.def_undef.range + &self.defines.iter()
            .map(|define| &define.range)
            .fold(self.def_undef.range.clone(), |sum, range| &sum + range)
    }
}

pub enum PragmaType {
    DecimalCommaAllowed,
}

pub struct Pragma {
    #[allow(dead_code)]
    pub pragma_type: PragmaType,
    pub range:Range,
}

pub struct PragmaExpr {
    pub pragma_token: Token,
    pub pragmas: Vec<Pragma>,
}

impl HasRange for PragmaExpr {
    fn get_range(&self) -> Range {
        &self.pragma_token.range + &self.pragmas.iter()
            .map(|pragma| &pragma.range)
            .fold(self.pragma_token.range.clone(), |sum, range| &sum + range)
    }
}


#[allow(unused)]
pub fn print_nodes(node: &Box<Node>, indent: usize, globals: &Globals) {
    print!("{: <1$}", "", indent);
    let indent= indent+5;
    match &node.expr {
        N::Const(expr) => {
            let value_str = match &expr.const_type {
                ConstType::Numeric { number } => number.to_double().to_string(),
                ConstType::FormattedString => globals.get_text(&expr.range).to_string()
            };
            println!("{0}: {1}{2}", "ConstExpr", value_str, node.unit.id);
        },
        N::Binary(expr) => {
            println!("{0}: {1:?}", "BinExpr", expr.op.kind);
            print_nodes(&expr.expr1, indent, globals);
            print_nodes(&expr.expr2, indent, globals);
        },
        N::None(expr) => {
            println!("{0}", "NoneExpr");
        },
        N::List(expr) => {
            println!("{0}", "ListExpr");
            for child in &expr.nodes {
                print_nodes(&child, indent, globals);
            }
        },
        N::Assign(expr) => {
            println!("{0}", "AssignExpr");
            print_nodes(&expr.expr, indent, globals);
        },
        N::Postfix(expr) => {
            println!("{0}", "PostfixExpr");
        },
        N::Comment(expr) => {
            println!("{0}", "CommentExpr");
        },
        N::Call(expr) => {
            println!("{0}", "CallExpr");
        },
        N::Id(expr) => {
            println!("{0}", "IdExpr");
        },
        N::FunctionDef(expr) => {
            println!("{0}", "FunctionDefExpr");
        },
        N::Unit(expr) => {
            println!("{0}", "UnitExpr");
        },
        N::Define(expr) => {
            println!("{0}", "DefineExpr");
        },
        N::Block(expr) => {
            println!("{0}", "CodeBlock");
            for stmt in &expr.statements {
                print_nodes(&stmt.node, indent, globals);
            }
        },
        _ => {
            println!("{0}", "It's a dunno...");
        }
    }
}

