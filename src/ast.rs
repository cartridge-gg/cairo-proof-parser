use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use starknet_types_core::felt::Felt;

#[derive(Debug, Clone)]
pub enum Expr {
    Value(String),
    Array(Vec<Expr>),
}
impl Into<Vec<Felt>> for Expr {
    fn into(self) -> Vec<Felt> {
        match self {
            Expr::Value(v) => vec![Felt::from_dec_str(&v).unwrap()],
            Expr::Array(v) => std::iter::once(Felt::from(v.len()))
                .chain(
                    v.into_iter()
                        .flat_map(|x| <Expr as Into<Vec<Felt>>>::into(x)),
                )
                .collect(),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Value(v) => write!(f, "{v}"),
            Expr::Array(v) => {
                write!(f, "{}", v.len())?;
                for expr in v.iter() {
                    write!(f, " {expr}")?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Exprs(pub Vec<Expr>);
impl Into<Vec<Felt>> for Exprs {
    fn into(self) -> Vec<Felt> {
        self.iter()
            .flat_map(|x| <Expr as Into<Vec<Felt>>>::into(x.to_owned()))
            .collect()
    }
}

impl Display for Exprs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;

        for (i, expr) in self.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{expr}")?;
        }

        write!(f, "]")?;

        Ok(())
    }
}

impl Deref for Exprs {
    type Target = Vec<Expr>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Exprs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
