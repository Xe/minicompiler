use std::error::Error;
use std::fmt;
use std::io;

/// Mathematical operations that our compiler can do.
#[derive(Debug, Eq, PartialEq)]
enum Op {
    Mul,
    Div,
    Add,
    Sub,
}

impl Op {
    fn precedence(&self) -> i32 {
        use Op::*;

        match self {
            Mul | Div => PREC_MUL,
            Add | Sub => PREC_ADD,
        }
    }
}

const PREC_EOF: i32 = 0;
const PREC_TERM: i32 = 1;
const PREC_MUL: i32 = 2;
const PREC_ADD: i32 = 3;
const PREC_PAREN: i32 = 4;

/// All of the possible tokens for the compiler, this limits the compiler
/// to simple math expressions.
#[derive(Debug, Eq, PartialEq)]
enum Token {
    EOF,
    Number(i32),
    Operation(Op),
    LeftParen,
    RightParen,
}

impl Token {
    fn precedence(&self) -> i32 {
        use Token::*;

        match self {
            EOF => PREC_EOF,
            Number(_) => PREC_TERM,
            Operation(op) => op.precedence(),
            LeftParen | RightParen => PREC_PAREN,
        }
    }
}

/// The error that gets returned on bad input. This only tells the user that it's
/// wrong because debug information is out of scope here. Sorry.
#[derive(Debug, Eq, PartialEq)]
struct BadInput;

// Errors need to be displayable.
impl fmt::Display for BadInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "something in your input is bad, good luck")
    }
}

// The default Error implementation will do here.
impl Error for BadInput {}

/// Turns a string of input into a slice of tokens. This goes over every character
/// in the string and combines numbers together.
fn lex(input: &str) -> Result<Vec<Token>, BadInput> {
    use Op::*;
    use Token::*;
    let mut result: Vec<Token> = Vec::new();

    for character in input.chars() {
        match character {
            // Skip whitespace
            ' ' => continue,

            // Ending characters
            ';' | '\n' => {
                result.push(EOF);
                break;
            }

            // Math operations
            '*' => result.push(Operation(Mul)),
            '/' => result.push(Operation(Div)),
            '+' => result.push(Operation(Add)),
            '-' => result.push(Operation(Sub)),

            // Parentheses
            '(' => result.push(LeftParen),
            ')' => result.push(RightParen),

            // Numbers
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                let num: i32 = (character as u8 - '0' as u8).into();
                if result.len() == 0 {
                    result.push(Number(num));
                    continue;
                }

                let last = result.pop().unwrap();

                match last {
                    Number(i) => {
                        result.push(Number((i * 10) + num));
                    }
                    _ => {
                        result.push(last);
                        result.push(Number(num));
                    }
                }
            }

            // Everything else is bad input
            _ => return Err(BadInput),
        }
    }

    Ok(result)
}

#[test]
fn basic_lexing() {
    assert!(lex("420 + 69").is_ok());
    assert_eq!(lex("420"), Ok(vec![Token::Number(420)]));
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input)?;

    let tokens = lex(input.as_str()).map_err(|why| io::Error::new(io::ErrorKind::Other, why))?;
    println!("{:#?}", tokens);

    Ok(())
}
