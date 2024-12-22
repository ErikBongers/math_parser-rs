use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum NumberFormat {
    Dec, Hex, Oct, Bin, Exp
}