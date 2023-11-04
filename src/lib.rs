use std::fs;

type Loc = (usize, usize);

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Keyword(String, Loc),
    Section(String, String, Loc),
    Integer(usize, Loc),
    Float(f64, Loc),
    Symbol(char, String, Loc),
    Ident(String, Loc),
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
pub struct Lexer {
    pub keywords: Vec<String>,
    pub sections: Vec<Section>,
    pub symbols: Vec<(char, String)>,
    pub buffer: Vec<u8>,
    pub allow_whitespace: bool,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub name: String,
    pub start: String,
    pub end: String,
}


impl Token {
    pub fn as_string(&self) -> String {
        return match self {
            Token::Keyword(keyword, _) => keyword.clone(),
            Token::Section(_, value, _) => value.clone(),
            Token::Integer(integer, _) => integer.to_string(),
            Token::Float(float, _) => float.to_string(),
            Token::Symbol(value, _, _) => value.to_string(),
            Token::Ident(ident, _) => ident.clone(),
        };
    }

    pub fn loc(&self) -> Loc {
        return match self {
            Token::Keyword(_, loc) => *loc,
            Token::Section(_, _, loc) => *loc,
            Token::Integer(_, loc) => *loc,
            Token::Float(_, loc) => *loc,
            Token::Symbol(_, _, loc) => *loc,
            Token::Ident(_, loc) => *loc,
        };
    }

    pub fn is_keyword(&self, keyword: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Token::Keyword(value, _) = self {
            if value == keyword {
                return Ok(());
            }
        }
        return Err(format!("expected keyword: {:?}", self).into());
    }

    pub fn is_section(&self, name: &str) -> Result<String, Box<dyn std::error::Error>> {
        if let Token::Section(s_name, value, _) = self {
            if name == s_name {
                return Ok(value.clone());
            }
        }
        return Err(format!("expected section: {:?}", self).into());
    }

    pub fn is_ident(&self) -> Result<String, Box<dyn std::error::Error>> {
        if let Token::Ident(value, _) = self {
            return Ok(value.clone());
        }
        return Err(format!("expected ident: {:?}", self).into());
    }

    pub fn is_integer(&self) -> Result<usize, Box<dyn std::error::Error>> {
        if let Token::Integer(integer, _) = self {
            return Ok(*integer);
        }
        return Err(format!("expected integer: {:?}", self).into());
    }

    pub fn is_float(&self) -> Result<f64, Box<dyn std::error::Error>> {
        if let Token::Float(float, _) = self {
            return Ok(*float);
        }
        return Err(format!("expected float: {:?}", self).into());
    }

    pub fn is_symbol(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Token::Symbol(_, s_name, _) = self {
            if s_name == name {
                return Ok(());
            }
        }
        return Err(format!("expected symbol: {:?}", self).into());
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

impl Lexer {
    pub fn new(keywords: &[String], sections: &[Section], symbols: &[(char, String)], allow_whitespace: bool) -> Lexer {
        return Lexer {
            keywords: keywords.to_vec(),
            sections: sections.to_vec(),
            symbols: symbols.to_vec(),
            buffer: Vec::new(),
            allow_whitespace,
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

    fn is_numeric(&self, token: &String, loc: Loc) -> Token {
        if let Ok(integer) = token.parse::<usize>() {
            return Token::Integer(integer, loc);
        } else if let Ok(integer) = token.parse::<f64>() {
            return Token::Float(integer, loc);
        } else {
            return Token::Ident(token.clone(), loc);
        }
    }

    fn lex_token(&self, token: &String, loc: Loc) -> Option<Token> {
        if token == "\n" {
            return None;
        } else if token == "" {
            if self.allow_whitespace {
                return Some(Token::Ident(" ".to_string(), loc));
            } else {
                return None;
            }
        } else if self.keywords.contains(&token) {
            return Some(Token::Keyword(token.clone(), loc));
        } else if token.len() == 1 {
            let character = token.chars().collect::<Vec<char>>()[0];
            if let Some(symbol_name) = self.symbols_contain(&character) {
                return Some(Token::Symbol(character, symbol_name.to_string(), loc));
            } else {
                return Some(self.is_numeric(token, loc));
            }
        } else if let Ok(name) = self.section_exists(&token[0..1], &token[token.len()-1..token.len()]) {
            return Some(Token::Section(name, token[1..token.len() - 1].to_string(), loc));
        } else {
            return Some(self.is_numeric(token, loc));
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, Box<dyn std::error::Error>> {
        if self.symbols_contain(&' ').is_none() {
            self.symbols.push((' ', "Space".to_string()));
        }

        let mut mode = Mode::Normal;
        let mut token = String::new();
        let mut tokens: Vec<Token> = Vec::new();
        let mut section: Vec<Section> = Vec::new();
        let mut loc = (1, 1);

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
                        self.lex_token(&token, loc).map(|t| tokens.push(t));
                        token = String::new();
                    } else if character.as_str() != " " {
                        token = token + &character;
                    }
                    if (self.symbols_contain(&char::from(byte.clone())).is_some() || self.symbols_contain(&char::from(self.buffer[index + 1])).is_some()) &&
                       section.len() == 0 { // making sure we arent lexing symbols when we're in a section
                        self.lex_token(&token, loc).map(|t| tokens.push(t));
                        token = String::new();
                    }
                } else if mode == Mode::Section {
                    if &character == "\\" {
                        if index + 1 >= self.buffer.len() {
                            return Ok(tokens);
                        } else {
                            index += 1;
                            token = token + &(self.buffer[index] as char).to_string();
                        }
                    } else if self.is_section(Value::End(section[0].start.to_string(), character.clone())).is_ok() || index + 2 >= self.buffer.len() { // index doesnt matter here because all indexes has the same start
                        println!("Closed");
                        token = token + &character;
                        self.lex_token(&token, loc).map(|t| tokens.push(t));
                        section = Vec::new();
                        token = String::new();
                        mode = Mode::Normal;
                    } else {
                        token = token + &character;
                    }
                }
            }

            if &character == "\n" {
                loc.0 += 1;
                loc.1 = 1;
            } else {
                loc.1 += 1;
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
            &["def".to_string(), "if".to_string(), "return".to_string()],
            &[Section::new("string", "\"", "\"")],
            &[(':', "column".to_string()), ('(', "openbrace".to_string()), (')', "closebrace".to_string())],
            true,
        );

        lexer.load_str("def test(): \" return 0 ");

        println!("tokens: {:?}", lexer.tokenize()?);
        return Ok(());
    }
}


