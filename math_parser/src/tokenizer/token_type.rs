#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    BracOpen,
    BracClose,
    CurlOpen,
    CurlClose,
    ParOpen,
    ParClose,
    Plus,
    Min,
    Div,
    Mult,
    Inc,
    Dec,

    //keep these together!
    Eq,
    EqPlus,
    EqMin,
    EqMult,
    EqDiv,
    EqUnit,
    //keep above together!

    Number,
    Power,
    Id,
    SemiColon,
    Comma,
    Dot, //except for the decimal dot.
    Ellipsis, //...
    Pipe,
    Exclam,
    EchoCommentLine,
    MuteLine,
    MuteStart,
    MuteEnd,
    QuotedStr,
    Newline,
    Function,
    Percent,
    Modulo,

    Define,
    Undef,
    Pragma,

    Unknown,
    Eot,
    ClearUnit, //dummy token used for "x=.;" where the unit is set to Unit::none
}