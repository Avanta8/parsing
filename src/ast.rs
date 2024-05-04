use std::fmt;

use crate::grammar::{NonTerminal, Terminal};

#[derive(Debug, Clone)]
pub enum AstNode<'a> {
    Terminal(&'a Terminal),
    NonTerminal(Ast<'a>),
}

#[derive(Debug, Clone)]
pub struct Ast<'a> {
    symbol: &'a NonTerminal,
    children: Vec<AstNode<'a>>,
}

impl<'a> AstNode<'a> {
    fn write_tree(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        match self {
            AstNode::Terminal(t) => write!(f, "{}", t.0)?,
            AstNode::NonTerminal(nt) => {
                write!(f, "{}", nt.symbol.0)?;
                for (i, child) in nt.children.iter().enumerate() {
                    write!(f, "{}", "\t".repeat(if i == 0 { 1 } else { level + 1 }))?;
                    child.write_tree(f, level + 1)?;
                    if i + 1 < nt.children.len() {
                        writeln!(f)?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl<'a> Ast<'a> {
    pub fn new(symbol: &'a NonTerminal, children: Vec<AstNode<'a>>) -> Self {
        Self { symbol, children }
    }
}
impl<'a> fmt::Display for Ast<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // HACK: Find a better way of doing this
        AstNode::NonTerminal(self.clone()).write_tree(f, 0)
    }
}
