use macros::define_errors;
use serde::Serialize;
use crate::tokenizer::cursor::Range;

#[derive(Clone, Serialize, PartialEq)]
pub enum ErrorType { E, W}

define_errors!(
    UnknownExpr: E : "Unknown expression `{expression}`.",
    NoOp: E : "No operator `{operator}` defined for `{operand1}` and `{operand2}`.",

    Eos: E : "Unexpected end of file.",

    VarNotDef: E : "variable `{variable}` is not defined.",
    VarNoValue: E : "variable `{variable}` has no value assigned.",
    FuncNotDef: E : "function `{function}` is not defined.",
    FuncNotAccessible: E : "function `{function}` is not accessible.",
    UnitNotDef: E : "unit `{unit}` is not defined.",
    DefineNotDef: E : "#define: option `{define_option}` not recognized.",
    UndefNotOk: E : "#undef: option `{define_option}` not recognized.",
    ExpHasUnit: E : "Exponent has a unit ({unit}).",

    FuncArgWrong: E : "Number of argument(s) for function `{function}` are wrong.",
    FuncArgWrongType: E : "Argument(s) for function `{function}` are of the wrong type. {type_info}",
    FuncNoBody: E : "Function body is empty for function `{function}`.",
    FuncNoOpenPar: E : "Missing `(` for function `{function}`.",
    FuncFailed: E : "Error in function `{function}`.",

    WDivImplMult: W : "Warning: ambiguous expression: division followed by implicit multiplication.",
    WPowImplMult: W : "Warning: ambiguous expression: exponential combined with implicit multiplication.",
    WAssumingUnit: W : "Warning: not all values have a unit. Assuming same unit as the other values.",
    WVarIsUnit: W : "Warning; variable `{variable}` overrides unit with same name.",
    VarIsUnit: E : "Variable `{variable}` overrides unit with same name.",
    WVarIsFunction: W : "Warning; variable `{variable}` overrides function with same name.",
    WFunctionOverride: W : "Warning; function `{function}` overrides an existing function.",
    FunctionOverride: E : "Function `{function}` overrides an existing function.",
    WUnitIsVar: W : "Warning: ambiguous implicit multiplication: `{variable_or_unit}` is both a variable and a unit.",
    WExplicitUnitsExpected: W : "Warning: explicit unit expected: `{units}`",
    EExplicitUnitsExpected: E : "Explicit unit expected: `{function}`",

    UnitPropDiff: E : "The units are not for the same property (lenght, temperature,...).",
    UnitPropWrong: E : "The units are not for the property {unit_property}.",
    ConstRedef: E : "Redefinition of constant `{constant}` not allowed.",
    WConstRedef: W : "Warning: redefinition of constant `{constant}`.",
    DateFragNoDate: E : "Cannot get `{fragment}` fragment. Value is not a date.",
    DateFragNoDuration: E : "Cannot get `{fragment}` fragment. Value is not a duration.",

    VarExpected: E : "The increment (++) or decrement (--) operator can only be used on a variable.",
    Expected: E : "Expected `{token}`.",
    ExpectedId: E : "Expected identifier.",
    ExpectedNumericValue: E : "Expected numeric value.",
    ValueError: E : "{error}",

    DateInvFrag: E : "Invalid fragment `{fragment}`for date.",
    DurInvFrag: E : "Invalid fragment `{fragment}` for duration.",
    InvDateStr: E : "Invalid date string: `{date_string_info}`",
    InvDateStrForFormat: E : "Invalid date string for format `{format}`",
    InvDateValue: E : "Invalid date value `{date_value}` for {date}.",
    InvDate: E : "Invalid date.",
    InvList: E : "Cannot convert value list. {extra_info}",
    InvNumberStr: E : "Invalid number string: `{number_string}`",
    InvFormat: E : "Invalid format for this type of value: `{value}`",
);

#[derive(Clone)]
pub struct Error {
    pub id: ErrorId,
    pub message: String, //fully expanded message with params
    pub error_type: ErrorType,
    pub range: Range, //don't make Option: source_index is needed to filter in GUI.
    pub stack_trace: Option<Vec<Error>>,
}

#[inline]
pub fn has_real_errors<'a, Iter>(errors: Iter) -> bool
    where Iter: IntoIterator<Item=&'a Error>
{
    errors.into_iter().find(|e| e.error_type == ErrorType::E).is_some()
}
