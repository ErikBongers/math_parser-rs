use std::collections::HashMap;
use once_cell::sync::Lazy;
use serde::Serialize;
use crate::tokenizer::cursor::Range;

#[derive(Eq, Hash, PartialEq)]
#[derive(Clone, Serialize)]
pub enum ErrorId
{
    None = 0,
    UnknownExpr,
    NoOp,
    Eos,

    VarNotDef,
    FuncNotDef,
    UnitNotDef,
    DefineNotDef,
    UndefNotOk,

    FuncArgWrong,
    FuncArgWrongType,
    FuncNoBody,
    FuncNoOpenPar,
    FuncFailed,
    UnitPropDiff,
    ConstRedef,
    ExpHasUnit,

    DateFragNoDate,
    DateFragNoDuration,
    DateInvFrag,
    DurInvFrag,

    VarExpected,
    Expected,
    ExpectedId,
    ExpectedNumericValue,

    InvDateStr,
    InvDateValue,
    InvList,
    InvNumberStr,

    WDivImplMult,
    WPowImplMult,
    WAssumingUnit,
    WVarIsUnit,
    WUnitIsVar,
    WVarIsFunction,
    WFunctionOverride,
    WExplicitUnitsExpected,
    EExplicitUnitsExpected,
}

#[derive(Serialize)]
pub enum ErrorType { E, W}

pub struct ErrorDef<'a> {
    pub id: ErrorId,
    pub error_type: ErrorType,
    pub name: &'a str,
    pub message: &'a str,
}

#[derive(Clone, Serialize)]
pub struct Error {
    pub id: ErrorId,
    pub message: String, //fully expanded message with params
    pub range: Range, //don't make Option: source_index is needed to filter in GUI.
    pub stack_trace: Option<Vec<Error>>,
}


pub static ERROR_MAP: Lazy<HashMap<ErrorId, ErrorDef>>  = Lazy::new(|| HashMap::from( [
(ErrorId::UnknownExpr, ErrorDef{id: ErrorId::UnknownExpr, error_type: ErrorType::E, name: "UnknownExpr", message: "Unknown expression.{0}"}),
(ErrorId::NoOp, ErrorDef{id: ErrorId::NoOp, error_type: ErrorType::E, name: "NoOp", message: "No operator `{0}` defined for `{1}` and `{2}`."}),
(ErrorId::Eos, ErrorDef{id: ErrorId::Eos, error_type: ErrorType::E, name: "EOS", message: "Unexpected end of file."}),

(ErrorId::VarNotDef, ErrorDef{id: ErrorId::VarNotDef, error_type: ErrorType::E, name: "VarNotDef", message: "variable `{0}` is not defined."}),
(ErrorId::FuncNotDef, ErrorDef{id: ErrorId::FuncNotDef, error_type: ErrorType::E, name: "FuncNotDef", message: "function `{0}` is not defined."}),
(ErrorId::UnitNotDef, ErrorDef{id: ErrorId::UnitNotDef, error_type: ErrorType::E, name: "UnitNotDef", message: "unit `{0}` is not defined."}),
(ErrorId::DefineNotDef, ErrorDef{id: ErrorId::DefineNotDef, error_type: ErrorType::E, name: "DefineNotDef", message: "#define: option `{0}`not recognized."}),
(ErrorId::UndefNotOk, ErrorDef{id: ErrorId::UndefNotOk, error_type: ErrorType::E, name: "UndefNotOk", message: "#undef: option `{0}` not recognized."}),
(ErrorId::ExpHasUnit, ErrorDef{id: ErrorId::ExpHasUnit, error_type: ErrorType::E, name: "ExpHasUnit", message: "Exponent has a unit ({0})."}),

(ErrorId::FuncArgWrong, ErrorDef{id: ErrorId::FuncArgWrong, error_type: ErrorType::E, name: "FuncArgWrong", message: "Number of argument(s) for function `{0}` are wrong."}),
(ErrorId::FuncArgWrongType, ErrorDef{id: ErrorId::FuncArgWrongType, error_type: ErrorType::E, name: "FuncArgWrongType", message: "Argument(s) for function `{0}` are off the wrong type."}),
(ErrorId::FuncNoBody, ErrorDef{id: ErrorId::FuncNoBody, error_type: ErrorType::E, name: "FuncNoBody", message: "Function body is empty for function `{0}`."}),
(ErrorId::FuncNoOpenPar, ErrorDef{id: ErrorId::FuncNoOpenPar, error_type: ErrorType::E, name: "FuncNoOpenPar", message: "Missing `(` for function `{0}`."}),
(ErrorId::FuncFailed, ErrorDef{id: ErrorId::FuncFailed, error_type: ErrorType::E, name: "FuncFailed", message: "Error in function `0}`."}),

(ErrorId::WDivImplMult, ErrorDef{id: ErrorId::WDivImplMult, error_type: ErrorType::W, name: "WDivImplMult", message: "Warning: ambiguous expression: division followed by implicit multiplication."}),
(ErrorId::WPowImplMult, ErrorDef{id: ErrorId::WPowImplMult, error_type: ErrorType::W, name: "WPowImplMult", message: "Warning: ambiguous expression: exponential combined with implicit multiplication."}),
(ErrorId::WAssumingUnit, ErrorDef{id: ErrorId::WAssumingUnit, error_type: ErrorType::W, name: "WAssumingUnit", message: "Warning: not all values have a unit. Assuming same unit as the other values."}),
(ErrorId::WVarIsUnit, ErrorDef{id: ErrorId::WVarIsUnit, error_type: ErrorType::W, name: "WVarIsUnit", message: "Warning; variable `{0}` overrides unit with same name."}),
(ErrorId::WVarIsFunction, ErrorDef{id: ErrorId::WVarIsFunction, error_type: ErrorType::W, name: "WVarIsFunction", message: "Warning; variable `{0}` overrides function with same name."}),
(ErrorId::WFunctionOverride, ErrorDef{id: ErrorId::WFunctionOverride, error_type: ErrorType::W, name: "WFunctionOverride", message: "Warning; function `{0}` overrides an existing function."}),
(ErrorId::WUnitIsVar, ErrorDef{id: ErrorId::WUnitIsVar, error_type: ErrorType::W, name: "WUnitIsVar", message: "Warning: ambiguous implicit multiplication: `{0}` is both a variable and a unit."}),
(ErrorId::WExplicitUnitsExpected, ErrorDef{id: ErrorId::WExplicitUnitsExpected, error_type: ErrorType::W, name: "WExplicitUnitsExpected", message: "Warning: explicit unit expected: `{0}`"}),
(ErrorId::EExplicitUnitsExpected, ErrorDef{id: ErrorId::EExplicitUnitsExpected, error_type: ErrorType::E, name: "EExplicitUnitsExpected", message: "Explicit unit expected: `{0}`"}),

(ErrorId::UnitPropDiff, ErrorDef{id: ErrorId::UnitPropDiff, error_type: ErrorType::E, name: "UnitPropDiff", message: "The units are not for the same property (lenght, temperature,...)."}),
(ErrorId::ConstRedef, ErrorDef{id: ErrorId::ConstRedef, error_type: ErrorType::E, name: "ConstRedef", message: "Warning: redefinition of constant `{0}`."}),
(ErrorId::DateFragNoDate, ErrorDef{id: ErrorId::DateFragNoDate, error_type: ErrorType::E, name: "DateFragNoDate", message: "Cannot get `{0}` fragment. Value is not a date."}),
(ErrorId::DateFragNoDuration, ErrorDef{id: ErrorId::DateFragNoDuration, error_type: ErrorType::E, name: "DateFragNoDuration", message: "Cannot get `{0}` fragment. Value is not a duration."}),

(ErrorId::VarExpected, ErrorDef{id: ErrorId::VarExpected, error_type: ErrorType::E, name: "VarExpected", message: "The increment (++) or decrement (--) operator can only be used on a variable."}),
(ErrorId::Expected, ErrorDef{id: ErrorId::Expected, error_type: ErrorType::E, name: "Expected", message: "Expected `{0}`."}),
(ErrorId::ExpectedId, ErrorDef{id: ErrorId::ExpectedId, error_type: ErrorType::E, name: "ExpectedId", message: "Expected identifier."}),
(ErrorId::ExpectedNumericValue, ErrorDef{id: ErrorId::ExpectedNumericValue, error_type: ErrorType::E, name: "ExpectedNumericValue", message: "Expected numeric value."}),

(ErrorId::DateInvFrag, ErrorDef{id: ErrorId::DateInvFrag, error_type: ErrorType::E, name: "DateInvFrag", message: "Invalid fragment `0}`for date."}),
(ErrorId::DurInvFrag, ErrorDef{id: ErrorId::DurInvFrag, error_type: ErrorType::E, name: "DurInvFrag", message: "Invalid fragment `{0}` for duration."}),
(ErrorId::InvDateStr, ErrorDef{id: ErrorId::InvDateStr, error_type: ErrorType::E, name: "InvDateStr", message: "Invalid date string: `{0}`"}),
(ErrorId::InvDateValue, ErrorDef{id: ErrorId::InvDateValue, error_type: ErrorType::E, name: "InvDateValue", message: "Invalid date value `{0}` for {1}."}),
(ErrorId::InvList, ErrorDef{id: ErrorId::InvList, error_type: ErrorType::E, name: "InvList", message: "Cannot convert value list. {0}"}),
(ErrorId::InvNumberStr, ErrorDef{id: ErrorId::InvNumberStr, error_type: ErrorType::E, name: "InvNumberStr", message: "Invalid number string: `{0}`"}),

]));

impl Error {
    pub fn build(id: ErrorId, range: Range, args: &[&str]) -> Error {
        let error_def = &ERROR_MAP[&id];
        let mut message_fmt = error_def.message.to_string();

        //format the message with x arguments.
        for (i, arg) in args.iter().enumerate() {
            let pattern = format!("{{{i}}}");
            message_fmt = message_fmt
                .replace(&pattern, arg);
        }

        Error{
            id,
            message: message_fmt,
            range,
            stack_trace: None
        }
    }
}