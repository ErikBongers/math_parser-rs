enum NumFormat { DEC, BIN, HEX }

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Whitespace,
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
    CommentLine,
    EchoCommentLine,
    EchoStart,
    EchoEnd,
    EchoDouble,
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

    Unknown,
    Eot,
    Nullptr
}