use std::fmt;


pub struct Tokenizer<Chars> {
    chars: Chars,
}

impl<Chars> Tokenizer<Chars> {
    pub fn new(chars: Chars) -> Self {
        Tokenizer {
            chars,
        }
    }
}

impl<Chars> Iterator for Tokenizer<Chars> where Chars: Iterator<Item=char> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let mut token = String::new();

        let start = self.chars.find(|c| !c.is_whitespace())?;

        if start == '"' {
            consume_string(&mut token, self.chars.by_ref());
            return Some(Token::String(token));
        }

        token.push(start);
        token.extend(
            self.chars
                .by_ref()
                .take_while(|c| !c.is_whitespace())
        );

        match token.as_str() {
            "[" => return Some(Token::QuoteOpen),
            "]" => return Some(Token::QuoteClose),

            _ => {
                if let Ok(number) = token.parse::<u32>() {
                    return Some(Token::Number(number));
                }

                return Some(Token::Word(token))
            }
        }
    }
}


fn consume_string<S>(token: &mut String, mut string: S)
    where S: Iterator<Item=char>
{
    let mut escape = false;

    while let Some(c) = string.next() {
        if escape {
            match c {
                'n' => {
                    token.push('\n');
                    escape = false;
                }
                c => {
                    panic!("Unexpected escape sequence: {}", c);
                }
            }
        }
        else {
            match c {
                '"'  => return,
                '\\' => escape = true,
                c    => token.push(c),
            }
        }
    }
}


#[derive(Clone)]
pub enum Token {
    Number(u32),
    QuoteOpen,
    QuoteClose,
    String(String),
    Word(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Number(number) => number.fmt(f),
            Token::QuoteOpen      => write!(f, "["),
            Token::QuoteClose     => write!(f, "]"),
            Token::String(string) => write!(f, "{}", string),
            Token::Word(word)     => write!(f, "{}", word),
        }
    }
}
