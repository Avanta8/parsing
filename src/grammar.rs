use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Terminal(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NonTerminal(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Symbol {
    Terminal(Terminal),
    NonTerminal(NonTerminal),
}

#[derive(Debug, Clone)]
pub struct Grammar {
    pub nonterminals: HashSet<NonTerminal>,
    pub terminals: HashSet<Terminal>,
    pub productions: Vec<Production>,
    pub start: NonTerminal,
}

impl Grammar {
    pub fn productions_from(&self, state: &NonTerminal) -> Vec<&Production> {
        self.productions
            .iter()
            .filter(|&p| &p.lhs == state)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Production {
    pub lhs: NonTerminal,
    pub rhs: Vec<Symbol>,
}
