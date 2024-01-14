pub mod value;
pub mod operator;
pub mod globals;
pub mod scope;
mod serialize;
pub mod unit;

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::rc::Rc;
use macros::CastAny;
use crate::errors::{Error, ErrorId};
use crate::functions::{FunctionDef, FunctionType};
use crate::parser::date::date::DateFormat;
use crate::parser::formatted_date_parser::parse_date_string;
use crate::parser::nodes::{AssignExpr, BinExpr, CallExpr, CodeBlock, CommentExpr, ConstExpr, ConstType, DefineExpr, FunctionDefExpr, HasRange, IdExpr, ListExpr, Node, PostfixExpr, Statement, UnaryExpr, UnitExpr};
use crate::parser::nodes::DefineType::{All, Arithm, Date, DateUnits, DecimalComma, DecimalDot, Dmy, Electric, Mdy, ShortDateUnits, Strict, Trig, Ymd};
use crate::resolver::globals::Globals;
use crate::resolver::operator::{operator_id_from, OperatorType};
use crate::resolver::scope::Scope;
use crate::resolver::unit::{Unit, UnitProperty, UnitsView, UnitTag};
use crate::resolver::value::{NumberFormat, Value, Variant};
use crate::resolver::value::Variant::Numeric;
use crate::tokenizer::cursor::{Number, Range};
use crate::tokenizer::token_type::TokenType;

pub struct Resolver<'g, 'a> {
    pub globals: &'g Globals,
    pub scope: Rc<RefCell<Scope>>,
    pub results: Vec<Value>,
    pub errors: &'a mut Vec<Error>,
    //date_format: DateFormat,
}

pub fn add_error(errors: &mut Vec<Error>, id: ErrorId, range: Range, args: &[&str], mut value: Value) -> Value {
    value.has_errors = true;
    errors.push(Error::build(id, range, args));
    value
}


impl<'g, 'a> Resolver<'g, 'a> {

    pub fn resolve(&mut self, statements: &Vec<Box<Statement>>) -> Option<Value> {
        for stmt in statements {
            let stmt = stmt.as_any().downcast_ref::<Statement>().unwrap();
            if TypeId::of::<DefineExpr>() == stmt.node.as_any().type_id() {
                self.resolve_define_expr(&stmt.node);
            } else {
                let mut result = self.resolve_node(&stmt.node);
                result.stmt_range = stmt.get_range();
                self.results.push(result);
            }
        };
        let Some(result) = self.results.last() else {
            return None
        };
        Some(result.clone())
    }

    pub fn add_error(&mut self, id: ErrorId, range: Range, args: &[&str]) {
        self.errors.push(Error::build(id, range, args));
    }

    pub fn return_error(&mut self, id: ErrorId, range: Range, args: &[&str], mut value: Value) -> Value {
        value.has_errors = true;
        self.add_error(id, range, args);
        value
    }

    pub fn add_error_value(&mut self, id: ErrorId, range: Range, args: &[&str]) -> Value {
        let mut value = Value::error(range.clone());
        value.has_errors = true;
        self.add_error(id, range, args);
        value
    }

    pub fn resolve_node(&mut self, expr: &Box<dyn Node>) -> Value {
        match expr.as_any().type_id() {
            t if TypeId::of::<CodeBlock>() == t => { self.resolve_codeblock_expr(expr) },
            t if TypeId::of::<ConstExpr>() == t => { self.resolve_const_expr(expr) },
            t if TypeId::of::<BinExpr>() == t => { self.resolve_bin_expr(expr) },
            t if TypeId::of::<IdExpr>() == t => { self.resolve_id_expr(expr) },
            t if TypeId::of::<AssignExpr>() == t => { self.resolve_assign_expr(expr) },
            t if TypeId::of::<UnaryExpr>() == t => { self.resolve_unary_expr(expr) },
            t if TypeId::of::<PostfixExpr>() == t => { self.resolve_postfix_expr(expr) },
            t if TypeId::of::<UnitExpr>() == t => { self.resolve_unit_expr(expr) },
            t if TypeId::of::<CallExpr>() == t => { self.resolve_call_expr(expr) },
            t if TypeId::of::<ListExpr>() == t => { self.resolve_list_expr(expr) },
            t if TypeId::of::<CommentExpr>() == t => { self.resolve_comment_expr(expr) },
            t if TypeId::of::<FunctionDefExpr>() == t => { self.resolve_func_def_expr(expr) },
            _ => self.add_error_value(ErrorId::Expected, expr.get_range(), &["Unknown expression to resolve_node"])
        }
    }

    fn resolve_codeblock_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let code_block = expr.as_any().downcast_ref::<CodeBlock>().unwrap();
        let mut resolver = Resolver {globals: self.globals, scope: code_block.scope.clone(), results: Vec::new(), errors: self.errors};
        let result = resolver.resolve(&code_block.statements);
        let Some(result) = result else {
            return self.add_error_value(ErrorId::FuncNoBody, code_block.get_range().clone(),&["anonymous block"]);
        };
        result
    }

    fn resolve_define_expr(&mut self, expr: &Box<dyn Node>) {
        let define_expr = expr.as_any().downcast_ref::<DefineExpr>().unwrap();
        if(define_expr.def_undef.kind == TokenType::Define) {
            self.resolve_defines(&define_expr);
        } else {
            self.resolve_undefines(&define_expr);
        }
    }

    fn resolve_defines(&mut self, define_expr: &DefineExpr) {
        for define in &define_expr.defines  {
            use crate::parser::nodes::DefineType::*;
            match &define.define_type {
                Ymd => self.scope.borrow_mut().date_format = DateFormat::YMD,
                Dmy => self.scope.borrow_mut().date_format = DateFormat::DMY,
                Mdy => self.scope.borrow_mut().date_format = DateFormat::MDY,
                Precision {ref number} => {
                    //TODO: test if integer!
                    self.scope.borrow_mut().precision = number.to_double() as i8;
                },
                DateUnits => self.scope.borrow_mut().units_view.add_tagged(&UnitTag::LongDateTime, self.globals),
                ShortDateUnits => self.scope.borrow_mut().units_view.add_tagged(&UnitTag::ShortDateTime, self.globals),
                Electric => {
                    self.scope.borrow_mut().units_view.add_class(&UnitProperty::VOLTAGE, &self.globals.unit_defs);
                    self.scope.borrow_mut().units_view.add_class(&UnitProperty::CURRENT, &self.globals.unit_defs);
                    self.scope.borrow_mut().units_view.add_class(&UnitProperty::RESISTANCE, &self.globals.unit_defs);
                },
                Strict => self.scope.borrow_mut().strict = true,
                DecimalDot => {
                    self.scope.borrow_mut().decimal_char = '.';
                    self.scope.borrow_mut().thou_char = ',';
                },
                DecimalComma => {
                    self.scope.borrow_mut().decimal_char = ',';
                    self.scope.borrow_mut().thou_char = '.';
                },
                Trig => self.scope.borrow_mut().function_view.add_type(FunctionType::Trig, self.globals),
                Arithm => self.scope.borrow_mut().function_view.add_type(FunctionType::Arithm, self.globals),
                Date => self.scope.borrow_mut().function_view.add_type(FunctionType::Date, self.globals),
                All => self.scope.borrow_mut().function_view.add_all(&self.globals.global_function_defs),
            }
        }
    }

    fn resolve_undefines(&mut self, define_expr: &DefineExpr) {
        for define in &define_expr.defines  {
            use crate::parser::nodes::DefineType::*;
            match &define.define_type {
                DateUnits => self.scope.borrow_mut().units_view.remove_tagged(UnitTag::LongDateTime, &self.globals.unit_defs),
                ShortDateUnits => self.scope.borrow_mut().units_view.remove_tagged(UnitTag::ShortDateTime, &self.globals.unit_defs),
                Electric => {
                    self.scope.borrow_mut().units_view.remove_class(&UnitProperty::VOLTAGE, self.globals);
                    self.scope.borrow_mut().units_view.remove_class(&UnitProperty::CURRENT, self.globals);
                    self.scope.borrow_mut().units_view.remove_class(&UnitProperty::RESISTANCE, self.globals);
                },
                Strict => self.scope.borrow_mut().strict = true,
                Trig => self.scope.borrow_mut().function_view.remove_type(FunctionType::Trig, self.globals),
                Arithm => self.scope.borrow_mut().function_view.remove_type(FunctionType::Arithm, self.globals),
                Date => self.scope.borrow_mut().function_view.remove_type(FunctionType::Date, self.globals),
                _ => self.add_error(ErrorId::UndefNotOk, define.range.clone(), &[self.globals.get_text(&define.range)]),
            }
        }
    }

    fn resolve_list_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let list_expr = expr.as_any().downcast_ref::<ListExpr>().unwrap();
        let mut number_list = Vec::<Value>::new();
        for item in &list_expr.nodes {
            let value = self.resolve_node(item);
            number_list.push(value);
        };
        Value {
            id: None,
            stmt_range: expr.get_range().clone(),
            variant: Variant::List {values: number_list},
            has_errors: false,
        }
    }

    fn resolve_comment_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        Value {
            id: None,
            stmt_range: expr.get_range().clone(),
            variant: Variant::Comment,
            has_errors: false,
        }
    }

    fn resolve_func_def_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let func_expr = expr.as_any().downcast_ref::<FunctionDefExpr>().unwrap();
        Value { //TODO: add id and full range of function.
            id: Some(func_expr.id_range.clone()),
            has_errors: false,
            stmt_range: func_expr.get_range(),
            variant: Variant::FunctionDef
        }
    }

    fn resolve_call_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let call_expr = expr.as_any().downcast_ref::<CallExpr>().unwrap();
        if call_expr.node_data.has_errors {
            return Value::error(call_expr.get_range());
        };
        let function_name = call_expr.function_name.as_str();
        //is function 'in view'?
        if self.scope.borrow().function_view.ids.contains(function_name) == false {
            //TODO: error: distinguish between non-existent function and function "out of view" or "not enabled".
            return self.add_error_value(ErrorId::FuncNotDef, call_expr.function_name_range.clone(), &[&call_expr.function_name]);
        }

        let global_function_def = self.globals.global_function_defs.contains_key(function_name);
        let local_function_def = self.scope.borrow().local_function_defs.contains_key(function_name);//TODO: also find function in parent scope.
        if !global_function_def && !local_function_def {
            return self.add_error_value(ErrorId::FuncNotDef, call_expr.function_name_range.clone(), &[&call_expr.function_name]);
        };

        let arguments = call_expr.arguments.as_any().downcast_ref::<ListExpr>().unwrap();

        //TODO: try trait objects. (trait references, actually). Doesn't work in combination with RefCell<Scope> and I haven't found a way to get rid of the RefCell.
        let mut arg_count_wrong = false;
        if global_function_def {
            if !self.globals.global_function_defs.get(function_name).unwrap().is_correct_arg_count(arguments.nodes.len()) {
                arg_count_wrong = true;
            }
        }
        //TODO: if both global and local function exists, then what? Use local?
        if local_function_def {
            if !self.scope.borrow().local_function_defs.get(function_name).unwrap().is_correct_arg_count(arguments.nodes.len()) {
                arg_count_wrong = true;
            }
        }

        if arg_count_wrong {
            return self.add_error_value(ErrorId::FuncArgWrong, call_expr.function_name_range.clone(), &[&call_expr.function_name]);
        };
        let mut arg_values: Vec<Value> = Vec::new();
        for arg in &arguments.nodes {
            let value = self.resolve_node(arg);
            if value.has_errors {
                return Value::error(value.stmt_range.clone());
            }
            arg_values.push(value);
        };

        let mut result = if global_function_def {
            self.globals.global_function_defs.get(function_name).unwrap()
                .call(&self.scope, &arg_values, &call_expr.function_name_range, &mut self.errors, self.globals)
        } else {
            if local_function_def {
                self.scope.borrow().local_function_defs.get(function_name).unwrap()
                    .call(&self.scope, &arg_values, &call_expr.function_name_range, &mut self.errors, self.globals)
            } else {
                panic!("TODO");
            }
        };
        Resolver::apply_unit(&mut result, expr, &self.scope.borrow().units_view, &expr.get_range(), self.errors, self.globals);
        result
    }

    fn resolve_unit_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let unit_expr = expr.as_any().downcast_ref::<UnitExpr>().unwrap();
        let mut result = self.resolve_node(&unit_expr.node);
        if let Numeric { .. } = &mut result.variant {
            Resolver::apply_unit(&mut result, expr, &self.scope.borrow().units_view, &expr.get_range(), self.errors, self.globals);
        }
        result
    }

    //A postfix is ALWAYS separated by a dot, contrary to a implcit mult or a 'glued' unit.
    //A 'glued' unit is applied to the variant itself (numeric, duration,...)
    fn resolve_postfix_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let pfix_expr = expr.as_any().downcast_ref::<PostfixExpr>().unwrap();
        let result = self.resolve_node(&pfix_expr.node);
        let id = self.globals.get_text(&pfix_expr.postfix_id.range).to_string();
        let mut result = match id.as_str() {
            "to_days" | "days" | "months" | "years" => self.resolve_duration_fragment(result, &id, &pfix_expr.postfix_id.range),
            "day" | "month" | "year" => self.resolve_date_fragment(&pfix_expr, result, &id),
            "bin" | "hex" | "dec" | "oct" | "exp" =>  self.resolve_num_format(pfix_expr, result, &id),
            _ => self.resolve_unit_postfix(result, &pfix_expr, &id)
        };

        Resolver::apply_unit(&mut result, expr, &self.scope.borrow().units_view, &expr.get_range(), self.errors, self.globals);
        result
    }

    fn resolve_unit_postfix(&mut self, mut result: Value, pfix_expr: &PostfixExpr, id: &String) -> Value {
        match &mut result.variant {
            Numeric { ref mut number, .. } => {
                if pfix_expr.postfix_id.kind == TokenType::ClearUnit {
                    number.unit = Unit::none();
                } else {
                    let postfix_id = self.globals.get_text(&pfix_expr.postfix_id.range);
                    let unit = if self.scope.borrow().var_exists(postfix_id, self.globals) {
                        Unit { range: Some(pfix_expr.postfix_id.range.clone()), id: self.scope.borrow().get_var(postfix_id, self.globals).as_number().unwrap().unit.id.clone() }
                    } else {
                        Unit { range: Some(pfix_expr.postfix_id.range.clone()), id: id.clone() }
                    };
                    number.convert_to_unit(&unit, &self.scope.borrow().units_view, &pfix_expr.postfix_id.range, self.errors, self.globals);
                }
            },
            _ => return self.return_error(ErrorId::UnknownExpr, pfix_expr.postfix_id.range.clone(), &["Postfix expression not valid here."], result)
        };
        result
    }
    fn resolve_num_format(&mut self, pfix_expr: &PostfixExpr, mut result: Value, id: &String) -> Value {
        if let Some(number) = result.as_number_mut() {
            number.fmt = match id.as_str() {
                "bin" => NumberFormat::Bin,
                "hex" => NumberFormat::Hex,
                "dec" => NumberFormat::Dec,
                "oct" => NumberFormat::Oct,
                "exp" => NumberFormat::Exp,
                _ => number.fmt.clone()
            }
        } else {
            return self.return_error(ErrorId::InvFormat, pfix_expr.postfix_id.range.clone(), &[id.as_str()], result);
        }
        result
    }

    fn resolve_date_fragment(&mut self, pfix_expr: &PostfixExpr, mut result: Value, id: &str) -> Value {
        let Some(date) = result.as_date() else {
            return self.return_error(ErrorId::InvFormat, pfix_expr.postfix_id.range.clone(), &[id], result);
        };
        let val = match id {
            "day" => date.day as i32,
            "year" => date.year,
            "month" => date.month.clone() as i32,
            _ => return result
        };
        Value {
            id: None,
            stmt_range: pfix_expr.postfix_id.range.clone(),
            variant: Variant::Numeric {
                number: Number {
                    significand: val as f64,
                    exponent: 0,
                    unit: Unit::none(),
                    fmt: NumberFormat::Dec,
                },
            },
            has_errors: false,
        }
    }

    fn resolve_duration_fragment(&mut self, mut result: Value, id: &str, range: &Range) -> Value {
        let Variant::Duration {mut duration} = result.variant else { return Value::error(range.clone())};
        duration.normalize();
        let value = match id {
            "days" => duration.days,
            "months" => duration.months,
            "years" => duration.years,
            "to_days" => duration.to_days(),
            _ => return Value::error(range.clone())
        };
        Value::from_number(Number { significand: value as f64, exponent: 0, unit: Unit::from_id(id), fmt: NumberFormat::Dec }, range.clone())
   }

    //in case of (x.km)m, both postfixId (km) and unit (m) are filled.
    fn apply_unit(value: &mut Value, node: &Box<dyn Node>, units_view: &UnitsView, range: &Range, errors: &mut Vec<Error>, globals: &Globals) {
        if let Some(number) = value.as_number_mut() {
            if !node.get_node_data().unit.is_empty() {
                number.convert_to_unit(&node.get_node_data().unit, units_view, range, errors, globals);
            }
        }
        //else: ignore.
    }

    fn resolve_assign_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let expr = expr.as_any().downcast_ref::<AssignExpr>().unwrap();
        let mut value = self.resolve_node(&expr.expr);
        let id_str = self.globals.get_text(&expr.id.range).to_string();
        if !self.scope.borrow().variables.contains_key(&id_str) {
            //TODO: test if id is function
            if self.globals.constants.contains_key(&id_str.as_str()) {
                self.add_error(ErrorId::ConstRedef, expr.id.range.clone(), &[id_str.as_str()]);
            }
            if self.scope.borrow().units_view.units.contains(&id_str) {
                self.add_error(ErrorId::WVarIsUnit, expr.id.range.clone(), &[id_str.as_str()]);
            }
            //TODO: disallow redefine of constant in case of `strict`
            self.scope.borrow_mut().variables.insert(id_str, value.clone());
        }
        value.id = Some(expr.id.range.clone()); //add id here to avoid adding id to the self.scope.variables.
        value
    }

    fn resolve_id_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let expr = expr.as_any().downcast_ref::<IdExpr>().unwrap();
        let id = self.globals.get_text(&expr.id.range).to_string();
        let var_exists = self.scope.borrow().var_exists(&id, self.globals);
        if var_exists  {
            self.scope.borrow().get_var(&id, self.globals).clone()
        } else {
            self.add_error_value(ErrorId::VarNotDef, expr.id.range.clone(), &[&id])
        }
    }

    fn resolve_unary_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let expr = expr.as_any().downcast_ref::<UnaryExpr>().unwrap();
        let mut result = self.resolve_node(&expr.expr);
        if expr.op.kind == TokenType::Min {
            if let Numeric {ref mut number,..} = result.variant {
                number.significand = -number.significand
            }
        }
        result
    }

    fn resolve_const_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let expr = expr.as_any().downcast_ref::<ConstExpr>().unwrap();
        match &expr.const_type {
            ConstType::Numeric { number } => {
                let mut n = number.clone();
                n.unit = expr.node_data.unit.clone();
                Value::from_number(n, expr.get_range())
           },
            ConstType::FormattedString => {
                //TODO: FormattedNumberParser
                let mut trimmed_range = expr.range.clone();
                if (trimmed_range.end - trimmed_range.start) >=2 {
                    trimmed_range.start += 1;
                    trimmed_range.end -= 1;
                }
                let string = self.globals.get_text(&trimmed_range);
                if string == "last" {
                    return Value::last_variant(expr.range.clone());
                }
                let mut date = parse_date_string(string, &expr.range, self.scope.borrow().date_format);
                self.errors.append(&mut date.errors);
                Value::from_date(date, expr.get_range())
            }
        }
    }
/*            {
            FormattedNumberParser numberParser;
            auto number = numberParser.parse(*codeBlock.scope, codeBlock.scope->getText(constExpr.value.range), constExpr.range());
            if (number.errors.empty())
                {
                return number;
                }

            FormattedDateParser dateParser;
            dateParser.dateFormat = this->dateFormat;
            auto date = dateParser.parse(codeBlock.scope->getText(constExpr.value.range), constExpr.range());
            if(!date.errors.empty())
                date.errors.insert(date.errors.begin(), number.errors.begin(), number.errors.end());
            return Value(date);
            }
*/
    fn resolve_bin_expr(&mut self, expr: &Box<dyn Node>) -> Value {
        let expr = expr.as_any().downcast_ref::<BinExpr>().unwrap();

        let error_cnt_before = self.errors.len();
        let expr1 = self.resolve_node(&expr.expr1);
        let expr2 = self.resolve_node(&expr.expr2);
        if error_cnt_before != self.errors.len() {
            for error in &self.errors[error_cnt_before..] {
                if error.id != ErrorId::None { //TODO: should be check if the error is a 'real' error and not a warning.
                    return Value::error(expr.get_range());
                }
            }
        }

        let operator_type = OperatorType::from(&expr.op.kind);
        let op_id = operator_id_from(&expr1.variant, operator_type, &expr2.variant);
        if !self.globals.exists_operator(op_id) {
            let op_str = operator_type.to_string();
            let val_type1 = &expr1.variant.name();
            let val_type2 = &expr2.variant.name();
            return self.add_error_value(ErrorId::NoOp, expr.get_range().clone(), &[&op_str, &val_type1, &val_type2]);
        }

        let args = vec![expr1, expr2];
        let range = Range { source_index: expr.get_range().source_index, start: 0, end: 0};

        let result = (self.globals.get_operator(op_id).unwrap())(&self.globals, &args, &range);
        if expr.is_implicit_mult() {
            if let Some(id_expr) = expr.expr2.as_any().downcast_ref::<IdExpr>() {
                let id_str = self.globals.get_text(&id_expr.id.range);
                if self.scope.borrow().units_view.units.contains(id_str) {
                    self.add_error_value(ErrorId::WUnitIsVar, id_expr.id.range.clone(), &[id_str]);
                }
            }
        }
        result
    }
}