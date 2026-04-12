use std::path::PathBuf;
pub struct DataPath<'a> {
    pub zip: &'a PathBuf,
    pub csv: &'a PathBuf,
    pub dir: &'a PathBuf,
}

#[derive(Debug)]
pub enum Value<'a> {
    Float(f64),
    Str(&'a str),
}
impl<'a> Value<'a> {
    pub fn get_string(&self) -> Option<&'a str> {
        if let Value::Str(s) = self {
            Some(s)
        } else {
            None
        }
    }
}
