use anyhow::Result;
use std::collections::HashMap;
use std::io::BufRead;

pub const STDOUT_PREFIX: &str = "__MF__";

pub type ValueExtractionStrategy = fn(&str) -> Option<Vec<u8>>;

pub struct Parser {
    parser_val_prefix: String,
    value_extractors: Vec<ValueExtractionStrategy>,
}

impl Parser {
    pub fn new(parser_val_prefix: String, value_extractors: Vec<ValueExtractionStrategy>) -> Self {
        Parser {
            parser_val_prefix,
            value_extractors,
        }
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

    fn extractors(&self) -> std::slice::Iter<'_, ValueExtractionStrategy>;

    ///
    /// Creates a map from a string using a regex with predefined attributes.
    ///
    fn parse_to_map(&self, reader: impl BufRead) -> Result<HashMap<String, Vec<u8>>> {
        let hm = reader
            .lines()
            .take_while(|r| r.is_ok())
            .map(|r| r.unwrap())
            .filter_map(|r| {
                if r.starts_with(self.val_prefix()) {
                    let mut extractors = self.extractors();
                    let sep = r.find(':')?;
                    let (k, val) = r.split_at(sep);
                    let val = val.strip_prefix(':')?;
                    let extracted: Vec<u8> = {
                        let val = val.trim();
                        let mut ext = None;
                        while let Some(e) = extractors.next() {
                            ext = e(val);
                            if ext.is_some() {
                                break;
                            }
                        }
                        ext
                    }?;

                    let result = (k.replace(self.val_prefix(), ""), extracted);
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

    fn extractors(&self) -> std::slice::Iter<'_, ValueExtractionStrategy> {
        self.value_extractors.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex::FromHex;
    use std::io::Cursor;

    fn default_extractors() -> Vec<ValueExtractionStrategy> {
        vec![|v| hex::decode(v).ok(), |v| Some(v.as_bytes().to_vec())]
    }

    #[async_std::test]
    async fn test_parser_parse_to_map_valid() {
        let p = Parser::new("__MF__".to_string(), default_extractors());
        let cursor = Cursor::new(b"lorem-ipsum");
        assert_eq!(p.parse_to_map(cursor).unwrap(), HashMap::new());

        // Output variables can be either hex-strings that will be decoded as bytes
        let mut hm = HashMap::new();
        hm.insert("hello".to_string(), hex::decode("5FFF").unwrap());
        let s: String = "hello_world:XX\n__MF nope: not good\n__MF__hello:5FFF".to_string();
        let cursor = Cursor::new(s);
        assert_eq!(p.parse_to_map(cursor).unwrap(), hm);

        let mut hm = HashMap::new();
        hm.insert(
            "body".to_string(),
            Vec::from_hex(hex::encode("hello")).unwrap(),
        );
        let s: String = format!("__MF__body:{}\nabcd", hex::encode("hello"));
        let cursor = Cursor::new(s);
        assert_eq!(p.parse_to_map(cursor).unwrap(), hm);

        // ... or regular strings for convenience
        let mut hm = HashMap::new();
        hm.insert("body".to_string(), b"blabla".to_vec());
        let s: String = format!("__MF__body:{}\nabcd", "blabla");
        let cursor = Cursor::new(s);
        assert_eq!(p.parse_to_map(cursor).unwrap(), hm);
    }
}
