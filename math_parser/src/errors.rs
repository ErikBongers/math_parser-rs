use macros::define_errors;
use serde::Serialize;
use crate::tokenizer::cursor::Range;

#[derive(Clone, Serialize, PartialEq)]
pub enum ErrorType { E, W}

define_errors!(
    UnknownExpr: "Unknown expression `{expression}`.",
    NoOp:  "No operator `{operator}` defined for `{operand1}` and `{operand2}`.",

    Eos: "Unexpected end of file.",

    VarNotDef: "variable `{variable}` is not defined.",
    FuncNotDef: "function `{function}` is not defined.",
    FuncNotAccessible: "function `{function}` is not accessible.",
    UnitNotDef: "unit `{unit}` is not defined.",
    DefineNotDef: "#define: option `{define_option}`not recognized.",
    UndefNotOk: "#undef: option `{define_option}` not recognized.",
    ExpHasUnit: "Exponent has a unit ({unit}).",

    FuncArgWrong: "Number of argument(s) for function `{function}` are wrong.",
    FuncArgWrongType: "Argument(s) for function `{function}` are of the wrong type. {type_info}",
    FuncNoBody: "Function body is empty for function `{function}`.",
    FuncNoOpenPar: "Missing `(` for function `{function}`.",
    FuncFailed: "Error in function `{function}`.",

    WDivImplMult: "Warning: ambiguous expression: division followed by implicit multiplication.",
    WPowImplMult: "Warning: ambiguous expression: exponential combined with implicit multiplication.",
    WAssumingUnit: "Warning: not all values have a unit. Assuming same unit as the other values.",
    WVarIsUnit: "Warning; variable `{variable}` overrides unit with same name.",
    VarIsUnit: "Variable `{variable}` overrides unit with same name.",
    WVarIsFunction: "Warning; variable `{variable}` overrides function with same name.",
    WFunctionOverride: "Warning; function `{function}` overrides an existing function.",
    FunctionOverride: "Function `{function}` overrides an existing function.",
    WUnitIsVar: "Warning: ambiguous implicit multiplication: `{variable_or_unit}` is both a variable and a unit.",
    WExplicitUnitsExpected: "Warning: explicit unit expected: `{units}`",
    EExplicitUnitsExpected: "Explicit unit expected: `{function}`",

    UnitPropDiff: "The units are not for the same property (lenght, temperature,...).",
    UnitPropWrong: "The units are not for the property {unit_property}.",
    ConstRedef: "Redefinition of constant `{constant}` not allowed.",
    WConstRedef: "Warning: redefinition of constant `{constant}`.",
    DateFragNoDate: "Cannot get `{fragment}` fragment. Value is not a date.",
    DateFragNoDuration: "Cannot get `{fragment}` fragment. Value is not a duration.",

    VarExpected: "The increment (++) or decrement (--) operator can only be used on a variable.",
    Expected: "Expected `{token}`.",
    ExpectedId: "Expected identifier.",
    ExpectedNumericValue: "Expected numeric value.",
    ValueError: "{error}",

    DateInvFrag: "Invalid fragment `{fragment}`for date.",
    DurInvFrag: "Invalid fragment `{fragment}` for duration.",
    InvDateStr: "Invalid date string: `{date_string_info}`",
    InvDateStrForFormat: "Invalid date string for format `{format}`",
    InvDateValue: "Invalid date value `{date_value}` for {date}.",
    InvDate: "Invalid date.",
    InvList: "Cannot convert value list. {extra_info}",
    InvNumberStr: "Invalid number string: `{number_string}`",
    InvFormat: "Invalid format for this type of value: `{value}`",
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
