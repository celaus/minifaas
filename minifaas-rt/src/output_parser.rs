use anyhow::Result;
use std::collections::HashMap;
use std::io::BufRead;

pub struct Parser {
    parser_val_prefix: String,
}

impl Parser {
    pub fn new(parser_val_prefix: String) -> Self {
        Parser { parser_val_prefix }
    }
}


///
/// Trait for a parser to parse input of the `std::io::Read` trait into a `HashMap<String, Vec<u8>>`;
/// 
/// ```rust,ignore
/// use std::io::Cursor;
/// let p = Parser::new("myprefix_".to_string());
/// let cursor = Cursor::new(b"something-without-a-prefix");
/// assert_eq!(p.parse_to_map(cursor).unwrap(), HashMap::new());
/// 
/// let mut hm = HashMap::new();
/// hm.insert("hello".to_string(), hex::decode("5FFF").unwrap());
/// let s: String = "hello_world:XX\nmyprefix_nope: not good\nmyprefix_hello:5FFF".to_string();
/// let cursor = Cursor::new(s);
/// assert_eq!(p.parse_to_map(cursor).unwrap(), hm);
/// ```
/// 
pub trait ReaderInput {
    fn val_prefix(&self) -> &str;

    fn parse_to_map(&self, reader: impl BufRead) -> Result<HashMap<String, Vec<u8>>> {
        let hm = reader
            .lines()
            .take_while(|r| r.is_ok())
            .map(|r| r.unwrap())
            .filter_map(|r| {
                if r.starts_with(self.val_prefix()) {
                    let sep = r.find(':')?;
                    let (k, val) = r.split_at(sep);
                    let val = val.strip_prefix(':')?;
                    let result = (
                        k.replace(self.val_prefix(), ""),
                        hex::decode(val.trim()).ok()?,
                    );
                    Some(result)
                } else {
                    None
                }
            })
            .collect();
        Ok(hm)
    }
}

impl ReaderInput for Parser {
    fn val_prefix(&self) -> &str {
        &self.parser_val_prefix
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::io::Cursor;

    #[async_std::test]
    async fn test_parser_parse_to_map_valid() {
        let p = Parser::new("__MF__".to_string());
        let cursor = Cursor::new(b"lorem-ipsum");
        assert_eq!(p.parse_to_map(cursor).unwrap(), HashMap::new());
        let mut hm = HashMap::new();
        hm.insert("hello".to_string(), hex::decode("5FFF").unwrap());
        let s: String = "hello_world:XX\n__MF nope: not good\n__MF__hello:5FFF".to_string();
        let cursor = Cursor::new(s);
        assert_eq!(p.parse_to_map(cursor).unwrap(), hm);
    }
}
