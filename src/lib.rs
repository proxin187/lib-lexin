use std::fs;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Keyword(String),
    Section(String, String),
    Integer(usize),
    Float(f64),
    Symbol(char, String),
    Ident(String),
}

#[derive(Debug)]
enum Value {
    Start(String),
    End(String, String),
}

enum StartOrSection<'a> {
    Start(Vec<String>),
    Section(&'a Section),
}

#[derive(PartialEq, Eq)]
enum Mode {
    Section,
    Normal,
}

#[derive(Debug)]
pub struct Lexer<'a> {
    pub keywords: Vec<&'a str>,
    pub sections: Vec<Section>,
    pub symbols: Vec<(char, String)>,
    pub buffer: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub name: String,
    pub start: String,
    pub end: String,
}

impl Token {
    pub fn is_keyword(&self, keyword: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Token::Keyword(value) = self {
            if value == keyword {
                return Ok(());
            }
        }
        return Err(format!("expected keyword: {:?}", self).into());
    }

    pub fn is_section(&self, name: &str) -> Result<String, Box<dyn std::error::Error>> {
        if let Token::Section(s_name, value) = self {
            if name == s_name {
                return Ok(value.clone());
            }
        }
        return Err(format!("expected section: {:?}", self).into());
    }

    pub fn is_ident(&self) -> Result<String, Box<dyn std::error::Error>> {
        if let Token::Ident(value) = self {
            return Ok(value.clone());
        }
        return Err(format!("expected ident: {:?}", self).into());
    }

    pub fn is_integer(&self) -> Result<usize, Box<dyn std::error::Error>> {
        if let Token::Integer(integer) = self {
            return Ok(*integer);
        }
        return Err(format!("expected integer: {:?}", self).into());
    }

    pub fn is_float(&self) -> Result<f64, Box<dyn std::error::Error>> {
        if let Token::Float(float) = self {
            return Ok(*float);
        }
        return Err(format!("expected float: {:?}", self).into());
    }

    pub fn is_symbol(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Token::Symbol(_, s_name) = self {
            if s_name == name {
                return Ok(());
            }
        }
        return Err(format!("expected float: {:?}", self).into());
    }
}

impl Section {
    pub fn new(name: &str, start: &str, end: &str) -> Section {
        return Section {
            name: name.to_string(),
            start: start.to_string(),
            end: end.to_string(),
        };
    }

    pub fn from_end(end: String) -> Section {
        return Section {
            name: String::new(),
            start: String::new(),
            end,
        };
    }
}

impl<'a> Lexer<'a> {
    fn symbols_to_string(symbols: &[(char, &'a str)]) -> Vec<(char, String)> {
        let mut vector: Vec<(char, String)> = Vec::new();
        for symbol in symbols {
            vector.push((symbol.0, symbol.1.to_string()));
        }
        return vector;
    }

    pub fn new(keywords: &[&'a str], sections: &[Section], symbols: &[(char, &'a str)]) -> Lexer<'a> {
        return Lexer {
            keywords: keywords.to_vec(),
            sections: sections.to_vec(),
            symbols: Lexer::symbols_to_string(symbols),
            buffer: Vec::new(),
        };
    }

    pub fn load_str(&mut self, string: &str) {
        self.buffer = string.as_bytes().to_vec();
    }

    pub fn load_file(&mut self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.buffer = fs::read(filename)?;
        return Ok(());
    }

    fn symbols_contain(&self, value: &char) -> Option<&str> {
        for symbol in &self.symbols {
            if symbol.0 == *value {
                return Some(&symbol.1);
            }
        }
        return None;
    }

    fn section_exists(&self, start: &str, end: &str) -> Result<String, ()> {
        for section in &self.sections {
            if section.start == start && section.end == end {
                return Ok(section.name.to_string());
            }
        }
        return Err(());
    }

    fn is_section(&self, value: Value) -> Result<StartOrSection, ()> {
        let mut matches: Vec<String> = Vec::new();
        for section in &self.sections {
            if let Value::Start(start) = &value {
                if &section.start == start {
                    matches.push(section.end.clone());
                }
            } else if let Value::End(start, end) = &value {
                if &section.end == end && &section.start == start {
                    return Ok(StartOrSection::Section(section)); // matches is not really needed here
                }
            }
        }

        if matches.len() != 0 {
            return Ok(StartOrSection::Start(matches));
        }
        return Err(());
    }

    fn is_numeric(&self, token: &String) -> Token {
        if let Ok(integer) = token.parse::<usize>() {
            return Token::Integer(integer);
        } else if let Ok(integer) = token.parse::<f64>() {
            return Token::Float(integer);
        } else {
            return Token::Ident(token.clone());
        }
    }

    fn lex_token(&self, token: &String) -> Option<Token> {
        if token != "\n" && token != "" {
            if self.keywords.contains(&token.as_str()) {
                return Some(Token::Keyword(token.clone()));
            } else if token.len() == 1 {
                let character = token.chars().collect::<Vec<char>>()[0];
                if let Some(symbol_name) = self.symbols_contain(&character) {
                    return Some(Token::Symbol(character, symbol_name.to_string()));
                } else {
                    return Some(self.is_numeric(token));
                }
            } else if let Ok(name) = self.section_exists(&token[0..1], &token[token.len()-1..token.len()]) {
                return Some(Token::Section(name, token[1..token.len() - 1].to_string()));
            } else {
                return Some(self.is_numeric(token));
            }
        }
        return None;
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, Box<dyn std::error::Error>> {
        if self.symbols_contain(&' ').is_none() {
            self.symbols.push((' ', "Space".to_string()));
        }

        let mut mode = Mode::Normal;
        let mut token = String::new();
        let mut tokens: Vec<Token> = Vec::new();
        let mut section: Vec<Section> = Vec::new();

        let mut index = 0;
        while index < self.buffer.len() {
            let byte = &self.buffer[index];
            let character = String::from_utf8(vec![byte.clone()])?;
            if (index + 1) < self.buffer.len() {
                if mode == Mode::Normal {
                    if let Ok(StartOrSection::Start(ends)) = self.is_section(Value::Start(character.clone())) {
                        token = token + &character;
                        for end in ends {
                            section.push(Section::from_end(end.clone()));
                            let idx = section.len() - 1;
                            section[idx].start = character.clone();
                        }
                        mode = Mode::Section;
                    } else if character.as_str() == "\n" {
                        self.lex_token(&token).map(|t| tokens.push(t));
                        token = String::new();
                    } else if character.as_str() != " " {
                        token = token + &character;
                    }
                    if (self.symbols_contain(&char::from(byte.clone())).is_some() || self.symbols_contain(&char::from(self.buffer[index + 1])).is_some()) &&
                       section.len() == 0 { // making sure we arent lexing symbols when we're in a section
                        self.lex_token(&token).map(|t| tokens.push(t));
                        token = String::new();
                    }
                } else if mode == Mode::Section {
                    if &character == "\\" {
                        index += 1;
                        if index >= self.buffer.len() {
                            return Ok(tokens);
                        } else {
                            token = token + &character;
                        }
                    } else if let Ok(_) = self.is_section(Value::End(section[0].start.to_string(), character.clone())) { // index doesnt matter here because all indexes has the same start
                        token = token + &character;
                        self.lex_token(&token).map(|t| tokens.push(t));
                        section = Vec::new();
                        token = String::new();
                        mode = Mode::Normal;
                    } else {
                        token = token + &character;
                    }
                }
            }
            index += 1;
        }
        return Ok(tokens);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_test() -> Result<(), Box<dyn std::error::Error>> {
        let mut lexer = Lexer::new(
            &["def", "if", "return"],
            &[Section::new("comment", "/*", "*/")],
            &[(':', "column"), ('(', "openbrace"), (')', "closebrace")],
        );
        lexer.load_str("def test(): return 0");

        let expected = [
            Token::Keyword("def".to_string()),
            Token::Ident("test".to_string()),
            Token::Symbol('(', "openbrace".to_string()),
            Token::Symbol(')', "closebrace".to_string()),
            Token::Symbol(':', "column".to_string()),
            Token::Keyword("return".to_string()),
            Token::Integer(0),
        ];

        let mut index = 0;
        for token in lexer.tokenize()? {
            assert_eq!(token, expected[index]);
            index += 1;
        }
        return Ok(());
    }
}


