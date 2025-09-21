use std::cmp::PartialEq;
use crate::compiler::lexer::Token;
use std::iter::Iterator;

pub struct Parser
{
    elements: Vec<CodeElement>,
    status: Status
}

pub fn parse(to_parse: impl Iterator<Item=Token>) -> Option<Vec<CodeElement>>
{
    let mut parser = Parser::new();
    parser.parse(to_parse);
    if parser.status != Status::Done
    {
        return None
    }
    Some(parser.code_elements())
}
impl Parser
{
    pub fn parse(&mut self, mut token_stream: impl Iterator<Item=Token>)
    {
        if self.status != Status::NotStarted
        {
            panic!("This parser already parsed a token stream!");
        }
        self.status = Status::Parsing;
        self.elements = parse_internal(&mut token_stream);
        if let None = token_stream.next()
        {
            self.status = Status::Done;
        }
        else
        {
            self.status = Status::Failed; //A loop was not closed
        }
    }

    pub fn new() -> Self
    {
        Self { elements: Vec::new(), status: Status::NotStarted }
    }


    pub fn code_elements(self) -> Vec<CodeElement>
    {
        if self.status != Status::Done
        {
            panic!("There are no elements!")
        }
        self.elements
    }

    pub fn status(&self) -> &Status
    {
        &self.status
    }
}

fn parse_internal(token_stream: &mut impl Iterator<Item=Token>) -> Vec<CodeElement>
{
    let mut code_elements = Vec::new();
    //A for would violate ownership in line 77
    while let Some(token) = token_stream.next()
    {
        match token
        {
            Token::Inc => code_elements.push(CodeElement::Inc),
            Token::Dec => code_elements.push(CodeElement::Dec),
            Token::IncPtr => code_elements.push(CodeElement::IncPtr),
            Token::DecPtr => code_elements.push(CodeElement::DecPtr),
            Token::Print => code_elements.push(CodeElement::Print),
            Token::Read => code_elements.push(CodeElement::Read),
            Token::LoopStart => code_elements.push(CodeElement::Loop(parse_internal(token_stream))),
            Token::LoopEnd => return code_elements //Loop closes, we are done
        }
    }
    //No more elements, we are done
    code_elements
}

#[derive(Debug)]
pub enum CodeElement
{
    IncPtr,
    DecPtr,
    Inc,
    Dec,
    Print,
    Read,
    Loop(Vec<CodeElement>)
}

#[derive(PartialEq)]
pub enum Status
{
    NotStarted,
    Parsing,
    Failed,
    Done
}