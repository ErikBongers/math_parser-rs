pub mod value;
pub mod operator;
pub mod scope;
mod serialize;
pub mod unit;
pub mod recursive_iterator;

use std::cell::RefCell;
use std::rc::Rc;
use crate::date::{DateFormat, Duration, parse_date_string};
use crate::errors;
use crate::errors::{Error, has_real_errors};
use crate::functions::FunctionType;
use crate::parser::nodes::{AssignExpr, BinExpr, CallExpr, CodeBlock, CommentExpr, ConstExpr, ConstType, DefineExpr, FunctionDefExpr, HasRange, IdExpr, ListExpr, Node, NodeType, PostfixExpr, Statement, UnaryExpr, UnitExpr};
use crate::globals::Globals;
use crate::number::{Number, parse_formatted_number};
use crate::resolver::operator::{operator_id_from, OperatorType};
use crate::resolver::scope::{DecimalChar, Scope};
use crate::resolver::unit::{Unit, UnitProperty, UnitsView, UnitTag};
use crate::resolver::value::{NumberFormat, Value, Variant};
use crate::resolver::value::Variant::Numeric;
use crate::tokenizer::cursor::Range;
use crate::tokenizer::token_type::TokenType;

pub struct Resolver<'g, 'a> {
    pub globals: &'g Globals,
    pub scope: Rc<RefCell<Scope>>,
    pub results: Vec<Value>,
    pub errors: &'a mut Vec<Error>,
    //date_format: DateFormat,
    pub muted: bool,
    pub current_statement_muted: bool,
}

pub fn add_error_value<'s>(errors: &mut Vec<Error>, error: Error) -> Value {
    let mut value = Value::error(error.range.clone());
    value.has_errors = true;
    errors.push(error);
    value
}

impl<'g, 'a> Resolver<'g, 'a> {

    pub fn resolve(&mut self, statements: &Vec<Statement>) -> Option<Value> {
        let mut last_result = None;

        for stmt in statements {
            let value = self.resolve_one_statement(stmt);
            if stmt.mute || self.muted {
                last_result = Some(value);
            } else {
                self.results.push(value);
                last_result = None; //this just sets a flag. Not that expensive.
            }
        };
        last_result.or(self.results.last().cloned())
    }

    pub fn resolve_to_result(&mut self, statements: &Vec<Statement>) -> Option<Value> {
        let mut result = None;
        for stmt in statements {
            result = Some(self.resolve_one_statement(stmt));
        };
        result
    }

    #[inline(always)]
    fn resolve_one_statement(&mut self, stmt: &Statement) -> Value {
        self.current_statement_muted = stmt.mute;
        let mut value = self.resolve_node(&stmt.node);
        value.stmt_range = stmt.get_range();
        value
    }

    pub fn return_error(&mut self, error: Error, mut value: Value) -> Value {
        value.has_errors = true;
        self.errors.push(error);
        value
    }

    pub fn add_error_value(&mut self, error: Error) -> Value {
        add_error_value(self.errors, error)
    }

    pub fn resolve_node(&mut self, node: &Box<Node>) -> Value {
        if node.has_errors {
            return Value::error(node.get_range());
        };
        match &node.expr {
            NodeType::Block(expr) => { self.resolve_codeblock_expr(expr) },
            NodeType::Const(expr) => { self.resolve_const_expr(expr, &node.unit) },
            NodeType::Binary(expr) => { self.resolve_bin_expr(expr, &node.unit) },
            NodeType::Id(expr) => { self.resolve_id_expr(expr, &node.unit) },
            NodeType::Assign(expr) => { self.resolve_assign_expr(expr) },
            NodeType::Unary(expr) => { self.resolve_unary_expr(expr) },
            NodeType::Postfix(expr) => { self.resolve_postfix_expr(expr, &node.unit) },
            NodeType::Unit(expr) => { self.resolve_unit_expr(expr, &node.unit) },
            NodeType::Call(expr) => { self.resolve_call_expr(expr, &node.unit) },
            NodeType::List(expr) => { self.resolve_list_expr(expr) },
            NodeType::Comment(expr) => { self.resolve_comment_expr(expr) },
            NodeType::FunctionDef(expr) => { self.resolve_func_def_expr(expr) },
            NodeType::Define(expr) => { self.resolve_define_expr(expr) },
            NodeType::Pragma(expr) => { Value::none(expr.get_range()) }, //TODO: check if pragma's need to be resolved.
            NodeType::None(expr) => { Value::none(expr.get_range()) },
        }
    }

    fn resolve_codeblock_expr(&mut self, code_block: &CodeBlock) -> Value {
        let mut resolver = Resolver {globals: self.globals, scope: code_block.scope.clone(), results: Vec::new(), errors: self.errors, muted: self.muted || self.current_statement_muted, current_statement_muted: false};
        let result = resolver.resolve(&code_block.statements);
        self.results.extend(resolver.results);
        let Some(mut result) = result else {
            return self.add_error_value(errors::func_no_body("anonymous block", code_block.get_range().clone()));
        };
        result.stmt_range = code_block.get_range();
        result
    }

    fn resolve_define_expr(&mut self, define_expr: &DefineExpr) -> Value {
        if define_expr.def_undef.kind == TokenType::Define {
            self.resolve_defines(&define_expr);
        } else {
            self.resolve_undefines(&define_expr);
        }
        Value {
            id: None,
            stmt_range: define_expr.get_range(),
            variant: Variant::Define,
            has_errors: false,
        }
    }

    fn resolve_defines(&mut self, define_expr: &DefineExpr) {
        for define in &define_expr.defines  {
            use crate::parser::nodes::DefineType as T;
            match &define.define_type {
                T::Ymd => self.scope.borrow_mut().date_format = DateFormat::YMD,
                T::Dmy => self.scope.borrow_mut().date_format = DateFormat::DMY,
                T::Mdy => self.scope.borrow_mut().date_format = DateFormat::MDY,
                T::Precision {ref number} => {
                    if ! number.is_int() {
                        self.errors.push(errors::expected("integer value, but found float", define.range.clone()));
                        continue;
                    }
                    self.scope.borrow_mut().precision = 10.0_f64.powf(number.to_double());
                },
                T::DateUnits => self.scope.borrow_mut().units_view.add_tagged(&UnitTag::LongDateTime, self.globals),
                T::ShortDateUnits => self.scope.borrow_mut().units_view.add_tagged(&UnitTag::ShortDateTime, self.globals),
                T::Electric => {
                    self.scope.borrow_mut().units_view.add_class(&UnitProperty::VOLTAGE, &self.globals.unit_defs);
                    self.scope.borrow_mut().units_view.add_class(&UnitProperty::CURRENT, &self.globals.unit_defs);
                    self.scope.borrow_mut().units_view.add_class(&UnitProperty::RESISTANCE, &self.globals.unit_defs);
                },
                T::Strict => self.scope.borrow_mut().strict = true,
                T::DecimalDot => {
                    self.scope.borrow_mut().decimal_char = DecimalChar::Dot;
                },
                T::DecimalComma => {
                    self.scope.borrow_mut().decimal_char = DecimalChar::Comma;
                },
                T::DecimalAuto => {
                    self.scope.borrow_mut().decimal_char = DecimalChar::Auto;
                },
                T::Trig => self.scope.borrow_mut().function_view.add_type(FunctionType::Trig, self.globals),
                T::Arithm => self.scope.borrow_mut().function_view.add_type(FunctionType::Arithm, self.globals),
                T::Date => self.scope.borrow_mut().function_view.add_type(FunctionType::Date, self.globals),
                T::Default => self.scope.borrow_mut().function_view.add_all(&self.globals.global_function_defs),
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
                _ => self.errors.push(errors::undef_not_ok(self.globals.get_text(&define.range), define.range.clone())),
            }
        }
    }

    fn resolve_list_expr(&mut self, list_expr: &ListExpr) -> Value {
        let mut value_list = Vec::<Value>::new();
        for item in &list_expr.nodes {
            let value = self.resolve_node(item);
            value_list.push(value);
        };
        let mut has_duration = false;
        let mut has_other = false;
        for value in &value_list {
            if let Numeric { number: num, ..} = &value.variant {
                has_duration |= self.globals.is_unit(&num.unit, UnitProperty::DURATION);
                has_other |= !self.globals.is_unit(&num.unit, UnitProperty::DURATION);
            }
        }
        if has_duration && !has_other {
            return self.resolve_duration_list(value_list, &list_expr);
        }
        Value {
            id: None,
            stmt_range: list_expr.get_range().clone(),
            variant: Variant::List {values: value_list},
            has_errors: false,
        }
    }

    fn resolve_duration_list(&mut self, value_list: Vec<Value>, list_expr: &ListExpr) -> Value {
        let mut has_days = false;
        let mut has_months = false;
        let mut has_years = false;
        let mut duration = Duration::new();
        for number in value_list.iter().map(|v| v.as_number().unwrap()) { //unwrap: already check by caller.
            match number.unit.id.as_str() {
                "days" => {
                    if has_days {
                        return self.add_error_value(errors::inv_list("Unit 'days' used more than once.", list_expr.get_range()));
                    }
                    has_days = true;
                    duration.days = number.to_double() as i32;
                }
                "months" => {
                    if has_months {
                        return self.add_error_value(errors::inv_list("Unit 'months' used more than once.", list_expr.get_range()));
                    }
                    has_months = true;
                    duration.months = number.to_double() as i32;
                }
                "years" => {
                    if has_years {
                        return self.add_error_value(errors::inv_list("Unit 'years' used more than once.", list_expr.get_range()));
                    }
                    has_years = true;
                    duration.years = number.to_double() as i32;
                }
                _ => { unreachable!("already checked")}
            }
        }
        Value::from_duration(duration, list_expr.get_range())
    }

    fn resolve_comment_expr(&mut self, comment_expr: &CommentExpr) -> Value {
        Value {
            id: None,
            stmt_range: comment_expr.get_range().clone(),
            variant: Variant::Comment,
            has_errors: false,
        }
    }

    fn resolve_func_def_expr(&mut self, function_def_expr: &FunctionDefExpr) -> Value {
        if self.scope.borrow().function_accessible(&function_def_expr.id) {
            if self.scope.borrow().strict {
                return self.add_error_value(errors::function_override(&function_def_expr.id, function_def_expr.get_range()));
            } else {
                self.errors.push(errors::w_function_override(&function_def_expr.id, function_def_expr.get_range()));
            }
        } else {
            self.scope.borrow_mut().function_view.ids.insert(function_def_expr.id.clone());
        }

        Value {
            id: Some(function_def_expr.id_range.clone()),
            has_errors: false,
            stmt_range: function_def_expr.get_range(),
            variant: Variant::FunctionDef
        }
    }


    fn resolve_call_expr(&mut self, call_expr: &CallExpr, unit: &Unit) -> Value {
        let function_name = call_expr.function_name.as_str();
        if self.scope.borrow().function_accessible(function_name) == false {
            let error = if self.scope.borrow().function_exists(function_name, self.globals) == false {
                errors::func_not_def(&call_expr.function_name, call_expr.function_name_range.clone())
            } else {
                errors::func_not_accessible(&call_expr.function_name, call_expr.function_name_range.clone())
            };
            return self.add_error_value(error);
        }

        //resolve the arguments.
        let mut arg_values: Vec<Value> = Vec::new();
        for arg in &call_expr.arguments {
            let value = self.resolve_node(arg);
            if value.has_errors {
                return Value::error(call_expr.get_range());
            }
            arg_values.push(value);
        };

        let Some(result) = self.scope.borrow().with_function(function_name, self.globals,|fd| {
            let mut args_ref = &arg_values;
            let mut exploded_args = Vec::new();
            if fd.get_min_args() > 1 {
                args_ref = explode_if_one_arg(&arg_values, &mut exploded_args);
            }

            if !fd.is_correct_arg_count(args_ref.len()) {
                return Err(errors::func_arg_wrong(&call_expr.function_name, call_expr.function_name_range.clone()));
            };
            let result = fd.call(&self.scope.clone(), args_ref,  &call_expr.function_name_range, &mut self.errors, self.globals);
            Ok(Resolver::apply_unit(result, unit, &self.scope.borrow().units_view, &call_expr.get_range(), self.errors, self.globals))
        }) else {
            return self.add_error_value(errors::func_not_def(&call_expr.function_name, call_expr.function_name_range.clone()));
        };
        result.unwrap_or_else(|error| {
            let error_value = Value::error(error.range.clone());
            self.errors.push(error);
            error_value
        })
    }

    fn resolve_unit_expr(&mut self, unit_expr: &UnitExpr, unit: &Unit) -> Value {
        let mut result = self.resolve_node(&unit_expr.node);
        if let Numeric { .. } = &mut result.variant {
            result = Resolver::apply_unit(result, unit, &self.scope.borrow().units_view, &unit_expr.get_range(), self.errors, self.globals)
        }
        result
    }

    //A postfix is ALWAYS separated by a dot, contrary to a implcit mult or a 'glued' unit.
    //A 'glued' unit is applied to the variant itself (numeric, duration,...)
    fn resolve_postfix_expr(&mut self, postfix_expr: &PostfixExpr, unit: &Unit) -> Value {
        let result = self.resolve_node(&postfix_expr.node);
        let id = self.globals.get_text(&postfix_expr.postfix_id.range).to_string();
        let result = match id.as_str() {
            "to_days" | "days" | "months" | "years" => self.resolve_duration_fragment(result, &id, &postfix_expr.postfix_id.range),
            "day" | "month" | "year" => self.resolve_date_fragment(&postfix_expr, result, &id),
            "bin" | "hex" | "dec" | "oct" | "exp" =>  self.resolve_num_format(postfix_expr, result, &id),
            _ => self.resolve_unit_postfix(result, &postfix_expr, &id)
        };

        Resolver::apply_unit(result, unit, &self.scope.borrow().units_view, &postfix_expr.get_range(), self.errors, self.globals)
    }

    fn resolve_unit_postfix(&mut self, mut result: Value, pfix_expr: &PostfixExpr, id: &String) -> Value {
        match &mut result.variant {
            Numeric { ref mut number, .. } => {
                if pfix_expr.postfix_id.kind == TokenType::ClearUnit {
                    number.unit = Unit::none();
                } else {
                    let unit = if let Some(var) = self.scope.borrow().variables.get(id) {
                        var.as_number().map_or(Unit::none(), |number| number.unit.clone())
                    } else {
                        if let Some(constant) = self.globals.constants.get(id.as_str()) {
                            constant.unit.clone()
                        } else {
                            Unit::from_id(&id, Some(pfix_expr.postfix_id.range.clone()))
                        }
                    };
                    number.convert_to_unit(&unit, &self.scope.borrow().units_view, &pfix_expr.postfix_id.range, self.errors, self.globals);
                }
            },
            _ => {
                let range = pfix_expr.postfix_id.range.clone();
                return self.return_error(errors::unknown_expr("Postfix expression not valid here.", range), result);
            }
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
            return self.return_error(errors::inv_format(id.as_str(), pfix_expr.postfix_id.range.clone()), result);
        }
        result
    }

    fn resolve_date_fragment(&mut self, pfix_expr: &PostfixExpr, mut result: Value, id: &str) -> Value {
        let Some(date) = result.as_date() else {
            return self.return_error(errors::inv_format(id, pfix_expr.postfix_id.range.clone()), result);
        };
        let val = match id {
            "day" => date.get_normalized_day() as i32,
            "year" => date.year.unwrap_or(0),
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

    fn resolve_duration_fragment(&mut self, result: Value, id: &str, range: &Range) -> Value {
        let Variant::Duration {mut duration} = result.variant else { return Value::error(range.clone())};
        duration.normalize();
        let value = match id {
            "days" => duration.days,
            "months" => duration.months,
            "years" => duration.years,
            "to_days" => duration.to_days(),
            _ => return Value::error(range.clone())
        };
        Value::from_number(Number { significand: value as f64, exponent: 0, unit: Unit::from_id(id, Some(range.clone())), fmt: NumberFormat::Dec }, range.clone())
   }

    //in case of (x.km)m, both postfixId (km) and unit (m) are filled.
    fn apply_unit(mut value: Value, unit: &Unit, units_view: &UnitsView, range: &Range, errors: &mut Vec<Error>, globals: &Globals) -> Value {
        if let Some(ref mut number) = &mut value.as_number_mut() {
            if !unit.is_empty() {
                number.convert_to_unit(unit, units_view, range, errors, globals);
            }
        }
        //else: ignore.
        value
    }

    fn resolve_assign_expr(&mut self, assign_expr: &AssignExpr) -> Value {
        let mut value = self.resolve_node(&assign_expr.expr);
        let id_str = self.globals.get_text(&assign_expr.id.range).to_string();
        if !self.scope.borrow().variables.contains_key(&id_str) {
            if self.scope.borrow().function_exists(&id_str, self.globals) {
                self.errors.push(errors::w_var_is_function(id_str.as_str(), assign_expr.id.range.clone()));
            }
            if self.scope.borrow().units_view.units.contains(&id_str) {
                if self.scope.borrow().strict {
                    return self.add_error_value(errors::var_is_unit(id_str.as_str(), assign_expr.id.range.clone()));
                } else {
                    self.errors.push(errors::w_var_is_unit(id_str.as_str(), assign_expr.id.range.clone()));
                }
            }
        }
        //disallow redefine of constant in case of `strict`. Error has already been added
        if self.globals.constants.contains_key(&id_str.as_str()) {
            if self.scope.borrow().strict {
                self.errors.push(errors::const_redef(id_str.as_str(), assign_expr.id.range.clone()));
                return value;
            } else {
                self.errors.push(errors::w_const_redef(id_str.as_str(), assign_expr.id.range.clone()));
            }
        }
        self.scope.borrow_mut().variables.insert(id_str, value.clone());
        value.id = Some(assign_expr.id.range.clone()); //add id here to avoid adding id to the self.scope.variables.
        value
    }

    fn resolve_id_expr(&mut self, id_expr: &IdExpr, unit: &Unit) -> Value {
        let id = self.globals.get_text(&id_expr.id.range).to_string();
        let var_exists = self.scope.borrow().variables.contains_key(&id);
        let result = if var_exists  {
            self.scope.borrow().variables[&id].clone()
        } else {
            if self.globals.constants.contains_key(id.as_str()) {
                Value::from_number(self.globals.constants[&id as &str].clone(), id_expr.get_range())
            } else {
                self.add_error_value(errors::var_not_def(&id, id_expr.id.range.clone()))
            }
        };
        Resolver::apply_unit(result, unit, &self.scope.borrow().units_view, &id_expr.get_range(), self.errors, self.globals)
    }

    fn resolve_unary_expr(&mut self, unary_expr: &UnaryExpr) -> Value {
        let mut result = self.resolve_node(&unary_expr.expr);
        if unary_expr.op.kind == TokenType::Min {
            if let Numeric {ref mut number,..} = result.variant {
                number.significand = -number.significand
            }
        }
        result
    }

    fn resolve_const_expr(&mut self, const_expr: &ConstExpr, unit: &Unit) -> Value {
        match &const_expr.const_type {
            ConstType::Numeric { number } => {
                let mut n = number.clone();
                n.unit = unit.clone();
                let res = Value::from_number(n, const_expr.get_range());
                Self::apply_unit(res, unit, &self.scope.borrow().units_view, unit.range.as_ref().unwrap_or(&const_expr.get_range()), self.errors, self.globals)
           },
            ConstType::FormattedString => {
                let num_error = match parse_formatted_number(self.globals.get_text(&const_expr.range), &const_expr.range, self.scope.borrow().decimal_char) {
                    Ok(number) => {
                       return Value::from_number(number, const_expr.range.clone());
                    }
                    Err(error) => {
                        error
                    }
                };

                let string = self.globals.get_text(&const_expr.range);
                if string == "last" {
                    return Value::last_variant(const_expr.range.clone());
                }
                let mut date = parse_date_string(string, &const_expr.range, self.scope.borrow().date_format);
                if date.errors.is_empty() == false {
                    self.errors.append(&mut date.errors);
                    //only add the num_error if the date parsing failed.
                    self.errors.push(num_error);
                }
                Value::from_date(date, const_expr.get_range())
            }
        }
    }

    fn resolve_bin_expr(&mut self, bin_expr: &BinExpr, unit: &Unit) -> Value {

        let error_cnt_before = self.errors.len();
        let expr1 = self.resolve_node(&bin_expr.expr1);
        let expr2 = self.resolve_node(&bin_expr.expr2);
        if error_cnt_before != self.errors.len() {
            if has_real_errors(&self.errors[error_cnt_before..]) {
                return Value::error(bin_expr.get_range());
            }
        }

        let operator_type = OperatorType::from(&bin_expr.op.kind);
        let op_id = operator_id_from(expr1.variant.to_operand_type(), operator_type, expr2.variant.to_operand_type());
        if !self.globals.exists_operator(op_id) {
            let op_str = operator_type.to_string();
            let val_type1 = &expr1.variant.name();
            let val_type2 = &expr2.variant.name();
            return self.add_error_value(errors::no_op(&op_str, &val_type1, &val_type2, bin_expr.get_range().clone()));
        }

        let args = vec![expr1, expr2];
        let range = Range { source_index: bin_expr.get_range().source_index, start: 0, end: 0};

        let result = (self.globals.get_operator(op_id).unwrap())(&self.globals, &args, &range, &mut self.errors); //unwrap: op_id already checked.
        if bin_expr.implicit_mult {
            if let NodeType::Id(id_expr) = &bin_expr.expr2.expr {
                let id_str = self.globals.get_text(&id_expr.id.range);
                if self.scope.borrow().units_view.units.contains(id_str) {
                    self.add_error_value(errors::w_unit_is_var(id_str, id_expr.id.range.clone()));
                }
            }
        }
        Resolver::apply_unit(result, unit, &self.scope.borrow().units_view, &bin_expr.get_range(), self.errors, self.globals)
    }
}

pub fn explode_if_one_arg<'a>(args: &'a Vec<Value>, exploded_args: &'a mut Vec<Value>) -> &'a Vec<Value> {
    if args.len() != 1 {
        return args;
    }

    if let Variant::List{values} = &args[0].variant {
        exploded_args.clone_from(values);
        exploded_args
    } else {
        args
    }
}
