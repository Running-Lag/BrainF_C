use std::str::FromStr;
use logos::{Lexer, Logos};

/*#[derive(Debug, Logos)]
// skip all non-op characters
#[logos(skip(".|\n", priority = 0))]
enum Token
{
    #[token("+")]
    Add,
    #[token("-")]
    Sub,
    #[token("*")]
    Mul,
    #[token("/")]
    Div,
    #[token("def")]
    Function,
    #[regex(r"\d+\.\d+", parse_decimal)]
    Decimal(f64),
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[token("{")]
    OpenCurlyBrace,
    #[token("}")]
    CloseCurlyBrace,
    #[token(",")]
    Comma,
    #[regex(r"[0-9a-z_]+")]
    String,
}*/

#[derive(Debug, Logos)]
// skip all non-op characters
#[logos(skip(".|\n", priority = 0))]
pub enum Token
{
    #[token("+")]
    Inc,
    #[token("-")]
    Dec,
    #[token(">")]
    IncPtr,
    #[token("<")]
    DecPtr,
    #[token(".")]
    Print,
    #[token(",")]
    Read,
    #[token("[")]
    LoopStart,
    #[token("]")]
    LoopEnd,
}

pub fn lex<'a>(input: &'a str) -> impl 'a+Iterator<Item=Token>
{
    Token::lexer(input).map(|token| token.unwrap())
}

fn parse_decimal(lex: &mut Lexer<Token>) -> Option<f64>
{
    f64::from_str(lex.slice()).ok()
}