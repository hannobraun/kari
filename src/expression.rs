use std::fmt;


#[derive(Clone, Debug)]
pub struct Expression {
    pub kind: Kind,
}


#[derive(Clone, Debug)]
pub enum Kind {
    Bool(Bool),
    Number(Number),
    List(List),
    String(String),
    Word(String),
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Kind::Bool(b)        => b.0.fmt(f),
            Kind::Number(number) => number.0.fmt(f),
            Kind::List(list)     => list.fmt(f),
            Kind::String(string) => string.fmt(f),
            Kind::Word(word)     => word.fmt(f),
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
            write!(f, "{} ", item.kind)?;
        }
        write!(f, "]")?;

        Ok(())
    }
}


pub trait Name {
    const NAME: &'static str;
}

pub trait Into {
    fn into_expression(self) -> Expression;
}

pub trait From : Sized {
    fn from_expression(expression: Expression) -> Result<Self, Expression>;
}


impl Name for Expression {
    const NAME: &'static str = "expression";
}

impl Into for Expression {
    fn into_expression(self) -> Expression {
        self
    }
}

impl From for Expression {
    fn from_expression(expression: Expression) -> Result<Self, Expression> {
        Ok(expression)
    }
}


macro_rules! impl_expression {
    ($($ty:ident, $name:expr;)*) => {
        $(
            impl Name for $ty {
                const NAME: &'static str = $name;
            }

            impl Into for $ty {
                fn into_expression(self) -> Expression {
                    Expression {
                        kind: Kind::$ty(self),
                    }
                }
            }

            impl From for $ty {
                fn from_expression(expression: Expression)
                    -> Result<Self, Expression>
                {
                    match expression.kind {
                        Kind::$ty(kind) => Ok(kind),
                        _               => Err(expression),
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
