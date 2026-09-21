use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Value { Null, Bool(bool), Int(i64), Float(f64), Str(String) }

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Null => Ok(()),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Int(i) => write!(f, "{i}"),
            Value::Float(x) => write!(f, "{x}"),
            Value::Str(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Debug, Default)]
pub struct Node {
    pub kind: String,                    // object, style, location, text
    pub name: String,                    // button, main ...
    pub props: HashMap<String, String>,  // id, style, width ...
    pub text: Option<String>,            // "строка" внутри блока
    pub children: Vec<Node>,
}

pub fn parse(src: &str) -> Result<Vec<Node>, String> {
    let mut p = P { c: src.chars().collect(), i: 0 };
    let mut out = vec![];
    loop {
        p.ws();
        if p.peek().is_none() { break; }
        let kind = p.ident();
        if kind.is_empty() { return Err(p.err("неожиданный символ")); }
        out.push(p.block(kind)?);
    }
    Ok(out)
}

/// Заменяет {name} на значение. \{ даёт обычную скобку.
pub fn fill(tpl: &str, get: impl Fn(&str) -> Option<Value>) -> String {
    let mut out = String::new();
    let mut it = tpl.chars();
    while let Some(c) = it.next() {
        match c {
            '\\' => match it.next() {
                Some('n') => out.push('\n'),
                Some(n) => out.push(n),
                None => {}
            },
            '{' => {
                let name: String = it.by_ref().take_while(|&c| c != '}').collect();
                if let Some(v) = get(name.trim()) { out.push_str(&v.to_string()); }
            }
            _ => out.push(c),
        }
    }
    out
}

struct P { c: Vec<char>, i: usize }

impl P {
    fn peek(&self) -> Option<char> { self.c.get(self.i).copied() }

    fn err(&self, msg: &str) -> String {
        let line = self.c[..self.i.min(self.c.len())].iter().filter(|&&c| c == '\n').count() + 1;
        format!("строка {line}: {msg}")
    }

    fn ws(&mut self) {
        loop {
            while self.peek().map_or(false, |c| c.is_whitespace()) { self.i += 1; }
            if self.peek() == Some('/') && self.c.get(self.i + 1) == Some(&'/') {
                while self.peek().map_or(false, |c| c != '\n') { self.i += 1; }
            } else { break; }
        }
    }

    fn ident(&mut self) -> String {
        let s = self.i;
        while self.peek().map_or(false, |c| c.is_alphanumeric() || c == '-' || c == '_') { self.i += 1; }
        self.c[s..self.i].iter().collect()
    }

    fn string(&mut self) -> String {
        self.i += 1; // открывающая "
        let mut s = String::new();
        while let Some(c) = self.peek() {
            self.i += 1;
            match c {
                '"' => break,
                '\\' => if let Some(n) = self.peek() {
                    self.i += 1;
                    if n != '"' { s.push('\\'); }
                    s.push(n);
                },
                _ => s.push(c),
            }
        }
        s
    }

    fn value(&mut self) -> String {
        let s = self.i;
        while self.peek().map_or(false, |c| !matches!(c, ';' | '}' | '\n')) { self.i += 1; }
        let v: String = self.c[s..self.i].iter().collect();
        if self.peek() == Some(';') { self.i += 1; }
        v.trim().to_string()
    }

    fn block(&mut self, kind: String) -> Result<Node, String> {
        let mut n = Node { kind, ..Default::default() };
        self.ws();
        if self.peek().map_or(false, |c| c != '{') { n.name = self.ident(); self.ws(); }
        if self.peek() != Some('{') { return Err(self.err("ожидалась {")); }
        self.i += 1;
        loop {
            self.ws();
            match self.peek() {
                None => return Err(self.err("не закрыта }")),
                Some('}') => { self.i += 1; return Ok(n); }
                Some('"') => n.text = Some(self.string()),
                _ => {
                    let word = self.ident();
                    if word.is_empty() { return Err(self.err("неожиданный символ")); }
                    self.ws();
                    if self.peek() == Some(':') {
                        self.i += 1;
                        self.ws();
                        let v = self.value();
                        n.props.insert(word, v);
                    } else {
                        n.children.push(self.block(word)?);
                    }
                }
            }
        }
    }
}