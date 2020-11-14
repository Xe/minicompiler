use std::error::Error;
use std::fmt;
use std::io;

/// Mathematical operations that our compiler can do.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
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

const PREC_MUL: i32 = 3;
const PREC_ADD: i32 = 2;

/// All of the possible tokens for the compiler, this limits the compiler
/// to simple math expressions.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum Token {
    EOF,
    Number(i32),
    Operation(Op),
    LeftParen,
    RightParen,
}

/// All possible parsing errors.
#[derive(Debug, Eq, PartialEq)]
pub enum ParsingError {
    BadInput,
    NoMatchingParen,
}

// Errors need to be displayable.
impl fmt::Display for ParsingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParsingError::BadInput => write!(f, "something in your input is bad, good luck"),
            ParsingError::NoMatchingParen => write!(f, "no matching paren found"),
        }
    }
}

// The default Error implementation will do here.
impl Error for ParsingError {}

impl Into<io::Error> for ParsingError {
    fn into(self) -> io::Error {
        io::Error::new(io::ErrorKind::Other, self)
    }
}

/// Turns a string of input into a slice of tokens. This goes over every character
/// in the string and combines numbers together.
fn lex(input: &str) -> Result<Vec<Token>, ParsingError> {
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
                let num: i32 = (character as u8 - '0' as u8) as i32;

                match result.pop() {
                    Some(Number(i)) => {
                        result.push(Number((i * 10) + num));
                    }
                    Some(last) => {
                        result.push(last);
                        result.push(Number(num));
                    }
                    None => {
                        result.push(Number(num));
                        continue;
                    }
                }
            }

            // Everything else is bad input
            _ => return Err(ParsingError::BadInput),
        }
    }

    Ok(result)
}

/// A Stack is a type that can peek at the first item in its collection.
trait Stack<T> {
    fn top(&self) -> Option<T>;
}

/// Implement Stack for std::vec::Vec so we can peek at the top token.
///
/// This returns a copy of the top of the Vec, so we need to have it be
/// a Vec of cloneable elements.
impl<T: Clone> Stack<T> for Vec<T> {
    fn top(&self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        self.get(self.len() - 1).map(|value| value.clone())
    }
}

fn tilt_until(operators: &mut Vec<Token>, output: &mut Vec<Token>, stop: Token) -> bool {
    while let Some(token) = operators.pop() {
        if token == stop {
            return true;
        }
        output.push(token)
    }
    false
}

/// Takes a list of Tokens and runs the [Shunting-yard](https://en.wikipedia.org/wiki/Shunting-yard_algorithm)
/// algorithm to turn infix notation into postfix notation.
fn parse(tokens: Vec<Token>) -> Result<Vec<Token>, ParsingError> {
    use Token::*;
    let mut result: Vec<Token> = vec![];
    let mut stack: Vec<Token> = vec![];

    for tok in tokens {
        match tok {
            Number(_) => result.push(tok),
            LeftParen => stack.push(tok),
            RightParen => {
                if !tilt_until(&mut stack, &mut result, LeftParen) {
                    return Err(ParsingError::NoMatchingParen);
                }
            }
            Operation(op) => {
                while let Some(top) = stack.top() {
                    match top {
                        LeftParen => break,
                        Operation(top_op) => {
                            let p = top_op.precedence();
                            let q = op.precedence();
                            if p > q {
                                result.push(stack.pop().unwrap());
                            } else {
                                break;
                            }
                        }
                        _ => unreachable!("{:?} must not be on the stack", top),
                    }
                }
                stack.push(tok);
            }
            EOF => break,
        }
    }

    if tilt_until(&mut stack, &mut result, LeftParen) {
        return Err(ParsingError::NoMatchingParen);
    }

    assert!(stack.is_empty());
    Ok(result)
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input)?;

    let tokens = lex(input.as_str()).map_err(|why| io::Error::new(io::ErrorKind::Other, why))?;
    let parsed_tokens = parse(tokens).map_err(|why| io::Error::new(io::ErrorKind::Other, why))?;
    println!("{:#?}", parsed_tokens);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Op::*, Token::*, *};

    #[test]
    fn basic_lexing() {
        assert!(lex("420 + 69").is_ok());
        assert!(lex("tacos are tasty").is_err());

        assert_eq!(
            lex("420 + 69"),
            Ok(vec![Number(420), Operation(Add), Number(69)])
        );
        assert_eq!(
            lex("(30 + 560) / 4"),
            Ok(vec![
                LeftParen,
                Number(30),
                Operation(Add),
                Number(560),
                RightParen,
                Operation(Div),
                Number(4)
            ])
        );
    }

    #[test]
    fn basic_parsing() {
        // things that should fail
        assert!(parse(vec![LeftParen]).is_err());
        assert!(parse(vec![RightParen]).is_err());

        // basic infix expression with parens
        assert_eq!(
            parse(vec![
                Number(3),
                Operation(Add),
                LeftParen,
                Number(4),
                Operation(Mul),
                Number(5),
                RightParen
            ])
            .unwrap(),
            vec![
                Number(3),
                Number(4),
                Number(5),
                Operation(Mul),
                Operation(Add),
            ],
        );
    }

    #[test]
    fn full_lex_parse() {
        // Go all the way with a more complicated expression
        let maybe_tree = parse(lex("3 + 4 * (420 - 69) / (2 + 4)").unwrap());
        assert!(maybe_tree.is_ok());

        let tree = maybe_tree.unwrap();
        assert_eq!(
            tree,
            vec![
                Number(3),
                Number(4),
                Number(420),
                Number(69),
                Operation(Sub),
                Number(2),
                Number(4),
                Operation(Add),
                Operation(Div),
                Operation(Mul),
                Operation(Add),
            ]
        )
    }
}
