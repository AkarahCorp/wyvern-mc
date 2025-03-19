use std::{iter::Peekable, str::Chars};

use super::{Nbt, NbtArray, NbtCompound};

pub struct StringReader<'a> {
    str: Peekable<Chars<'a>>,
    idx: usize,
}

impl<'a> StringReader<'a> {
    pub fn new(str: &'a str) -> Self {
        StringReader {
            str: str.chars().peekable(),
            idx: 0,
        }
    }

    pub fn read(&mut self) -> Option<char> {
        self.idx += 1;
        self.str.next()
    }

    pub fn expect(&mut self, char: char) {
        self.skip_whitespace();
        self.idx += 1;
        let ch = self.str.next();
        assert_eq!(ch, Some(char));
    }

    pub fn peek(&mut self) -> Option<&char> {
        self.str.peek()
    }

    pub fn skip(&mut self) {
        self.str.next();
    }

    pub fn skip_whitespace(&mut self) {
        while self.peek().unwrap_or(&'0').is_whitespace() {
            self.skip();
        }
    }

    pub fn read_identifier(&mut self) -> Option<String> {
        let mut str = String::new();
        loop {
            match self.peek()? {
                '"' => {
                    self.skip();
                    while *self.peek()? != '"' {
                        str.push(self.read()?);
                    }
                    self.skip();
                    return Some(str);
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    str.push(self.read()?);
                }
                _ => {
                    break;
                }
            }
        }
        Some(str)
    }
}

pub struct SNbtParser;

impl SNbtParser {
    pub fn parse(nbt: &str) -> Option<Nbt> {
        let mut reader = StringReader::new(nbt);
        Self::parse_from_reader(&mut reader)
    }

    pub fn parse_from_reader(reader: &mut StringReader<'_>) -> Option<Nbt> {
        let r = {
            reader.skip_whitespace();
            let ch = reader.peek()?;
            match ch {
                '{' => {
                    let mut compound = NbtCompound::new();
                    reader.expect('{');
                    loop {
                        reader.skip_whitespace();
                        if *reader.peek()? == '}' {
                            reader.skip();
                            break;
                        }
                        reader.skip_whitespace();
                        let name = reader.read_identifier()?;
                        reader.skip_whitespace();
                        reader.expect(':');
                        let value = Self::parse_from_reader(reader)?;
                        compound.set(name, value);

                        match reader.read()? {
                            '}' => break,
                            ',' => continue,
                            _ => return None,
                        }
                    }
                    Some(Nbt::Compound(compound))
                }
                '[' => {
                    let mut array = NbtArray::new();
                    reader.expect('[');
                    loop {
                        reader.skip_whitespace();
                        if *reader.peek()? == ']' {
                            reader.skip();
                            break;
                        }
                        reader.skip_whitespace();
                        let value = Self::parse_from_reader(reader)?;
                        array.push(value).ok()?;
                        reader.skip_whitespace();
                        match reader.read()? {
                            ']' => break,
                            ',' => continue,
                            _ => return None,
                        }
                    }
                    Some(Nbt::Array(array))
                }
                '-' | '0'..='9' | '.' => {
                    let mut num_str = String::new();
                    while matches!(*reader.peek()?, '-' | '0'..='9' | '.') {
                        num_str.push(reader.read()?);
                    }
                    let wl = &num_str[0..num_str.len() - 1];
                    match num_str.chars().last()? {
                        'b' | 'B' => return Some(Nbt::Byte(wl.parse().ok()?)),
                        's' | 'S' => return Some(Nbt::Short(wl.parse().ok()?)),
                        'i' | 'I' => return Some(Nbt::Int(wl.parse().ok()?)),
                        'l' | 'L' => return Some(Nbt::Long(wl.parse().ok()?)),

                        'f' | 'F' => return Some(Nbt::Float(wl.parse().ok()?)),
                        'd' | 'D' => return Some(Nbt::Double(wl.parse().ok()?)),
                        _ => {}
                    }
                    if num_str.contains(".") {
                        return Some(Nbt::Double(num_str.parse().ok()?));
                    }
                    Some(Nbt::Int(num_str.parse().ok()?))
                }
                '"' => {
                    reader.expect('"');
                    let mut str = String::new();
                    loop {
                        match *reader.peek()? {
                            '"' => break,
                            '\\' => {
                                reader.skip();
                                reader.skip();
                            }
                            _ => str.push(reader.read()?),
                        }
                    }
                    reader.expect('"');
                    Some(Nbt::String(str))
                }
                '\'' => {
                    reader.expect('\'');
                    let mut str = String::new();
                    loop {
                        match *reader.peek()? {
                            '\'' => break,
                            '\\' => {
                                reader.skip();
                                reader.skip();
                            }
                            _ => str.push(reader.read()?),
                        }
                    }
                    reader.expect('\'');
                    Some(Nbt::String(str))
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let ident = reader.read_identifier()?;
                    match ident.as_str() {
                        "true" => Some(Nbt::Boolean(true)),
                        "false" => Some(Nbt::Boolean(false)),
                        _ => None,
                    }
                }
                _ => None,
            }
        };
        r
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        compound, list,
        nbt::{Nbt, SNbtParser},
    };

    #[test]
    pub fn compounds() {
        let snbt = "{}";
        let equiv = Nbt::Compound(compound! {});
        let nbt = SNbtParser::parse(snbt);
        assert_eq!(nbt, Some(equiv));
    }

    #[test]
    pub fn compounds_keyed() {
        let snbt = "{inner:{}}";
        let equiv = Nbt::Compound(compound! {
            inner: compound! {}
        });
        let nbt = SNbtParser::parse(snbt);
        assert_eq!(nbt, Some(equiv));
    }

    #[test]
    pub fn nums() {
        let snbt = "{x:10}";
        let equiv = Nbt::Compound(compound! {
                x: 10
        });
        let nbt = SNbtParser::parse(snbt);
        assert_eq!(nbt, Some(equiv));
    }
    #[test]
    pub fn multiple_nums() {
        let snbt = "{x:10, y:20, z:30}";
        let equiv = Nbt::Compound(compound! {
            x: 10, y: 20, z: 30
        });
        let nbt = SNbtParser::parse(snbt);
        assert_eq!(nbt, Some(equiv));
    }

    #[test]
    pub fn strings() {
        let snbt = "{name:\"AUsername\", other_str:'AAA'}";
        let equiv = Nbt::Compound(compound! {
            name: "AUsername", other_str: "AAA"
        });
        let nbt = SNbtParser::parse(snbt);
        assert_eq!(nbt, Some(equiv));
    }

    #[test]
    pub fn list() {
        let snbt = "[1, 2, 3, 4]";
        let equiv = Nbt::Array(list![1, 2, 3, 4]);
        let nbt = SNbtParser::parse(snbt);
        assert_eq!(nbt, Some(equiv));
    }

    #[test]
    pub fn bools() {
        let snbt = "[true, false, false, true]";
        let equiv = Nbt::Array(list![true, false, false, true]);
        let nbt = SNbtParser::parse(snbt);
        assert_eq!(nbt, Some(equiv));
    }
}
