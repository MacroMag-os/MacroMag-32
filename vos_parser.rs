use crate::vos_core::{
    CompareOp, SandboxConfig, VosCommand, VosPipeline, VosTransform, VosValue,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Ident(String),
    StringLit(String),
    IntLit(i64),
    BytesLit(u64),
    BoolLit(bool),
    Dot,        // .
    Comma,      // ,
    Colon,      // :
    Pipe,       // |
    Arrow,      // ->
    LParen,     // (
    RParen,     // )
    Eq,         // ==
    Neq,        // !=
    Gt,         // >
    Lt,         // <
    Gte,        // >=
    Lte,        // <=
    Sandbox,    // sandbox
    Where,      // where
    Select,     // select
    Limit,      // limit
    Eof,
}

/// 1. LEXER: Zamienia surowy ciąg znaków na strumień tokenów
pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }

        match ch {
            '.' => { chars.next(); tokens.push(Token::Dot); }
            ',' => { chars.next(); tokens.push(Token::Comma); }
            ':' => { chars.next(); tokens.push(Token::Colon); }
            '|' => { chars.next(); tokens.push(Token::Pipe); }
            '(' => { chars.next(); tokens.push(Token::LParen); }
            ')' => { chars.next(); tokens.push(Token::RParen); }
            '-' => {
                chars.next();
                if chars.peek() == Some(&'>') {
                    chars.next();
                    tokens.push(Token::Arrow);
                } else {
                    return Err("Oczekiwano '>' po '-'".into());
                }
            }
            '=' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Eq);
                } else {
                    return Err("Oczekiwano '=='".into());
                }
            }
            '!' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Neq);
                } else {
                    return Err("Nieznany operator '!'".into());
                }
            }
            '>' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Gte);
                } else {
                    tokens.push(Token::Gt);
                }
            }
            '<' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Lte);
                } else {
                    tokens.push(Token::Lt);
                }
            }
            '"' => {
                chars.next(); // pomijamy otwierający cudzysłów
                let mut val = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '"' {
                        chars.next();
                        break;
                    }
                    val.push(c);
                    chars.next();
                }
                tokens.push(Token::StringLit(val));
            }
            c if c.is_ascii_digit() => {
                let mut num_str = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() {
                        num_str.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let num: i64 = num_str.parse().map_err(|_| "Błędna liczba")?;

                // Sprawdzamy jednostki rozmiaru (np. 512MB, 2GB)
                let mut unit = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_alphabetic() {
                        unit.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }

                if unit.is_empty() {
                    tokens.push(Token::IntLit(num));
                } else {
                    let multiplier = match unit.to_uppercase().as_str() {
                        "B" => 1,
                        "KB" => 1024,
                        "MB" => 1024 * 1024,
                        "GB" => 1024 * 1024 * 1024,
                        _ => return Err(format!("Nieznana jednostka: {}", unit)),
                    };
                    tokens.push(Token::BytesLit((num as u64) * multiplier));
                }
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let mut ident = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        ident.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let tok = match ident.as_str() {
                    "sandbox" => Token::Sandbox,
                    "where" => Token::Where,
                    "select" => Token::Select,
                    "limit" => Token::Limit,
                    "true" => Token::BoolLit(true),
                    "false" => Token::BoolLit(false),
                    _ => Token::Ident(ident),
                };
                tokens.push(tok);
            }
            _ => return Err(format!("Nieoczekiwany znak: {}", ch)),
        }
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}

/// 2. PARSER: Buduje drzewo AST (VosPipeline) na podstawie tokenów
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn next(&mut self) -> Token {
        let tok = self.peek().clone();
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        let tok = self.next();
        if tok == expected {
            Ok(())
        } else {
            Err(format!("Oczekiwano {:?}, otrzymano {:?}", expected, tok))
        }
    }

    pub fn parse_pipeline(&mut self) -> Result<VosPipeline, String> {
        let sandbox = if self.peek() == &Token::Sandbox {
            Some(self.parse_sandbox()?)
        } else {
            None
        };

        let source = self.parse_command()?;
        let mut transforms = Vec::new();

        while self.peek() == &Token::Pipe {
            self.next(); // konsumujemy '|'
            transforms.push(self.parse_transform()?);
        }

        self.expect(Token::Eof)?;

        Ok(VosPipeline {
            sandbox,
            source,
            transforms,
        })
    }

    fn parse_sandbox(&mut self) -> Result<SandboxConfig, String> {
        self.expect(Token::Sandbox)?;
        self.expect(Token::LParen)?;

        let mut config = SandboxConfig {
            allow_net: false,
            read_only_fs: true,
        };

        while self.peek() != &Token::RParen {
            let key = match self.next() {
                Token::Ident(s) => s,
                other => return Err(format!("Oczekiwano nazwy parametru sandboxa, jest {:?}", other)),
            };
            self.expect(Token::Colon)?;
            let val = match self.next() {
                Token::BoolLit(b) => b,
                other => return Err(format!("Oczekiwano true/false, jest {:?}", other)),
            };

            match key.as_str() {
                "net" => config.allow_net = val,
                "ro_fs" => config.read_only_fs = val,
                _ => return Err(format!("Nieznana opcja sandboxa: {}", key)),
            }

            if self.peek() == &Token::Comma {
                self.next();
            }
        }

        self.expect(Token::RParen)?;
        self.expect(Token::Arrow)?;
        Ok(config)
    }

    fn parse_command(&mut self) -> Result<VosCommand, String> {
        let module = match self.next() {
            Token::Ident(s) => s,
            other => return Err(format!("Oczekiwano nazwy modułu, jest {:?}", other)),
        };
        self.expect(Token::Dot)?;
        let action = match self.next() {
            Token::Ident(s) => s,
            other => return Err(format!("Oczekiwano nazwy akcji, jest {:?}", other)),
        };
        self.expect(Token::LParen)?;

        let mut args = Vec::new();
        while self.peek() != &Token::RParen {
            args.push(self.parse_value()?);
            if self.peek() == &Token::Comma {
                self.next();
            }
        }
        self.expect(Token::RParen)?;

        Ok(VosCommand { module, action, args })
    }

    fn parse_transform(&mut self) -> Result<VosTransform, String> {
        match self.next() {
            Token::Where => {
                let field = match self.next() {
                    Token::Ident(s) => s,
                    other => return Err(format!("Oczekiwano nazwy pola w 'where', jest {:?}", other)),
                };
                let op = match self.next() {
                    Token::Eq => CompareOp::Eq,
                    Token::Neq => CompareOp::Neq,
                    Token::Gt => CompareOp::Gt,
                    Token::Lt => CompareOp::Lt,
                    Token::Gte => CompareOp::Gte,
                    Token::Lte => CompareOp::Lte,
                    other => return Err(format!("Nieznany operator porównania: {:?}", other)),
                };
                let value = self.parse_value()?;
                Ok(VosTransform::Where { field, op, value })
            }
            Token::Select => {
                let mut fields = Vec::new();
                loop {
                    match self.next() {
Token::Ident(s) => fields.push(s),
other => return Err(format!("Oczekiwano nazwy pola w 'select', jest {:?}", other)),
}
if self.peek() == &Token::Comma {
self.next();
} else {
break;
}
}
Ok(VosTransform::Select(fields))
}
Token::Limit => match self.next() {
Token::IntLit(n) if n >= 0 => Ok(VosTransform::Limit(n as usize)),
other => Err(format!("'limit' wymaga dodatniej liczby całkowitej, jest {:?}", other)),
},
other => Err(format!("Nieznana transformacja w potoku: {:?}", other)),other => Err(format!("Nieznana transformacja w potoku: {:?}", other)),
}
}
fn parse_value(&mut self) -> Result<VosValue, String> {
match self.next() {
Token::StringLit(s) => Ok(VosValue::Text(s)),
Token::IntLit(n) => Ok(VosValue::Int(n)),
Token::BytesLit(b) => Ok(VosValue::Bytes(b)),
Token::BoolLit(b) => Ok(VosValue::Bool(b)),
other => Err(format!("Oczekiwano wartości, otrzymano {:?}", other)),
}
}
}
