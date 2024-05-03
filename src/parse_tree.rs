use std::fmt;

use crate::grammar::{NonTerminal, Terminal};

#[derive(Debug, Clone)]
pub enum ParseTree<'a> {
    Terminal(&'a Terminal),
    NonTerminal(&'a NonTerminal, Vec<ParseTree<'a>>),
}

impl<'a> ParseTree<'a> {
    fn write_tree(&self, f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
        match self {
            ParseTree::Terminal(t) => write!(f, "{}", t.0)?,
            ParseTree::NonTerminal(nt, children) => {
                write!(f, "{}", nt.0)?;
                for (i, child) in children.iter().enumerate() {
                    write!(f, "{}", "\t".repeat(if i == 0 { 1 } else { level + 1 }))?;
                    child.write_tree(f, level + 1)?;
                    if i + 1 < children.len() {
                        writeln!(f)?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl<'a> fmt::Display for ParseTree<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_tree(f, 0)
    }
}
