use std::fmt;

#[derive(Debug, Clone)]
pub enum Variant {
    Int(i32),
    Float(f32),
    Bool(bool),
    String(String),
}

impl Variant {
    #[allow(dead_code)]
    pub fn as_int(&self) -> i32 {
        if let &Self::Int(v) = self {
            return v;
        }
        panic!();
    }

    #[allow(dead_code)]
    pub fn as_float(&self) -> f32 {
        if let &Self::Float(v) = self {
            return v;
        }
        panic!();
    }

    #[allow(dead_code)]
    pub fn as_bool(&self) -> bool {
        if let &Self::Bool(v) = self {
            return v;
        }
        panic!();
    }

    #[allow(dead_code)]
    pub fn as_string(&self) -> String {
        if let Self::String(v) = self {
            return v.clone();
        }
        panic!();
    }
}

impl fmt::Display for Variant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(v) => {
                write!(f, "{}", v)
            }
            Self::Float(v) => {
                write!(f, "{}", v)
            }
            Self::Bool(v) => {
                write!(f, "{}", v)
            }
            Self::String(v) => {
                write!(f, "{}", v)
            }
        }
    }
}

#[derive(Clone)]
pub struct Arg {
    pub name: String,
    pub value: Variant,
}

impl Arg {
    #[allow(dead_code)]
    pub fn int(name: &str, value: i32) -> Self {
        Self {
            name: name.to_string(),
            value: Variant::Int(value),
        }
    }

    #[allow(dead_code)]
    pub fn float(name: &str, value: f32) -> Self {
        Self {
            name: name.to_string(),
            value: Variant::Float(value),
        }
    }

    #[allow(dead_code)]
    pub fn bool(name: &str, value: bool) -> Self {
        Self {
            name: name.to_string(),
            value: Variant::Bool(value),
        }
    }

    #[allow(dead_code)]
    pub fn string(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: Variant::String(value.to_string()),
        }
    }
}
