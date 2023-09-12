[![Stars](https://img.shields.io/github/stars/proxin187/lib-lexin.svg?style=for-the-badge)](https://github.com/lib-lexin/argin/stargazers)
[![Forks](https://img.shields.io/github/forks/proxin187/lib-lexin.svg?style=for-the-badge)](https://github.com/lib-lexin/argin/forks)

# Lib-lexin

A simple lexer library.

<li>
    <a href="#Description">Description</a>
</li>
<li>
    <a href="#getting-started">Getting Started</a>
    <ul>
    <li><a href="#Usage">Usage</a></li>
    <li><a href="#Example">Example</a></li>
    <li><a href="#Functions">Functions</a></li>
    </ul>
</li>
<li><a href="#Authors">Authors</a></li>
<li><a href="#Versions">Versions</a></li>
<li><a href="#License">License</a></li>

## Description

lib-lexin is a small lexer library created to be able to quickly lex anything

## Getting Started

### Usage


#### Example
This example shows how easy it is to lex a file
```rust
use lib_lexin::{Lexer, Section};

let mut lexer = Lexer::new(
    &[ // keywords
        "fn",
        "return"
    ],
    &[ // section
        Section::new("string", "\"", "\""),
    ],
    &[ // symbols
        ('+', "Plus"),
    ],
);

lexer.load_file("[FILE]");

let tokens = lexer.tokenize()?;
```

#### Functions

Lexer::new
```rust
pub fn new(keywords: &[&'a str], sections: &[Section], symbols: &[(char, &'a str)]) -> Lexer<'a>
```
Lexer::load_file
```rust
pub fn load_file(&mut self, filename: &str) -> Result<(), Box<dyn std::error::Error>>
```
Lexer::load_str
```rust
pub fn load_str(&mut self, string: &str)
```
Lexer::tokenize
```rust
pub fn tokenize(&mut self) -> Result<Vec<Token>, Box<dyn std::error::Error>>
```
Section::new
```rust
pub fn new(name: &str, start: &str, end: &str) -> Section
```
Token::is_keyword
```rust
pub fn is_keyword(&self, keyword: &str) -> Result<(), Box<dyn std::error::Error>>
```
Token::is_section
```rust
pub fn is_section(&self, name: &str) -> Result<String, Box<dyn std::error::Error>>
```
Token::is_ident
```rust
pub fn is_ident(&self) -> Result<String, Box<dyn std::error::Error>>
```
Token::is_integer
```rust
    pub fn is_integer(&self) -> Result<usize, Box<dyn std::error::Error>>
```
Token::is_float
```rust
pub fn is_float(&self) -> Result<f64, Box<dyn std::error::Error>>
```
Token::is_symbol
```rust
pub fn is_symbol(&self, name: &str) -> Result<(), Box<dyn std::error::Error>>
```

## Help

Sections are always escaped

## Authors

Contributors names and contact info

* [Proxin](https://github.com/proxin187)

## Versions

* 0.1
    * Initial Release

## License

Currently there is no license, this may change in the future


