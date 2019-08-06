use std::fmt;


#[derive(Clone, Debug)]
pub struct Expression {
    pub data: Data,
}


#[derive(Clone, Debug)]
pub enum Data {
    Bool(Bool),
    Number(Number),
    List(List),
    String(String),
    Word(String),
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Data::Bool(b)        => b.0.fmt(f),
            Data::Number(number) => number.0.fmt(f),
            Data::List(list)     => list.fmt(f),
            Data::String(string) => string.fmt(f),
            Data::Word(word)     => word.fmt(f),
        }
    }
}


#[derive(Clone, Debug)]
pub struct Bool(pub bool);


#[derive(Clone, Debug)]
pub struct Number(pub u32);


#[derive(Clone, Debug)]
pub struct List(pub Vec<Expression>);

impl List {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl IntoIterator for List {
    type Item     = <Vec<Expression> as IntoIterator>::Item;
    type IntoIter = <Vec<Expression> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[ ")?;
        for item in &self.0 {
            write!(f, "{} ", item.data)?;
        }
        write!(f, "]")?;

        Ok(())
    }
}


pub trait Kind {
    const NAME: &'static str;

    fn to_expression(self) -> Expression;
}

impl Kind for Expression {
    const NAME: &'static str = "expression";

    fn to_expression(self) -> Expression {
        self
    }
}

macro_rules! impl_expression {
    ($($ty:ident, $name:expr;)*) => {
        $(
            impl Kind for $ty {
                const NAME: &'static str = $name;

                fn to_expression(self) -> Expression {
                    Expression {
                        data: Data::$ty(self),
                    }
                }
            }
        )*
    }
}

impl_expression!(
    Bool,   "bool";
    List,   "list";
    Number, "number";
    String, "string";
);
