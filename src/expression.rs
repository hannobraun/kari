use std::fmt;


#[derive(Clone, Debug)]
pub struct Expression {
    pub data: ExpressionKind,
}


#[derive(Clone, Debug)]
pub enum ExpressionKind {
    Bool(Bool),
    Number(Number),
    List(List),
    String(String),
    Word(String),
}

impl fmt::Display for ExpressionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExpressionKind::Bool(b)        => b.0.fmt(f),
            ExpressionKind::Number(number) => number.0.fmt(f),
            ExpressionKind::List(list)     => list.fmt(f),
            ExpressionKind::String(string) => string.fmt(f),
            ExpressionKind::Word(word)     => word.fmt(f),
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


pub trait ToExpression {
    fn to_expression(self) -> Expression;
}

impl ToExpression for Expression {
    fn to_expression(self) -> Expression {
        self
    }
}

macro_rules! impl_expression {
    ($($name:ident;)*) => {
        $(
            impl ToExpression for $name {
                fn to_expression(self) -> Expression {
                    Expression {
                        data: ExpressionKind::$name(self),
                    }
                }
            }
        )*
    }
}

impl_expression!(
    Bool;
    Number;
    List;
    String;
);
