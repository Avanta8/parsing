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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Production {
    lhs: NonTerminal,
    rhs: Vec<Symbol>,
}

impl Production {
    pub fn lhs(&self) -> &NonTerminal {
        &self.lhs
    }

    pub fn rhs(&self) -> &[Symbol] {
        &self.rhs
    }
}

#[derive(Debug, Clone)]
pub struct Grammar {
    nonterminals: HashSet<NonTerminal>,
    terminals: HashSet<Terminal>,
    productions: Vec<Production>,
    start: NonTerminal,
}

impl Grammar {
    pub fn nonterminals(&self) -> &HashSet<NonTerminal> {
        &self.nonterminals
    }

    pub fn terminals(&self) -> &HashSet<Terminal> {
        &self.terminals
    }

    pub fn productions(&self) -> &[Production] {
        &self.productions
    }

    pub fn productions_from(&self, state: &NonTerminal) -> Vec<&Production> {
        self.productions
            .iter()
            .filter(|&p| &p.lhs == state)
            .collect()
    }

    pub fn start(&self) -> &NonTerminal {
        &self.start
    }
}

pub fn build_grammar(
    given_nonterminals: &str,
    terminals: &str,
    // given_productions: Vec<(impl Into<String>, Vec<Vec<impl Into<String>>>)>,
    given_productions: Vec<(&str, &str)>,
    start: &str,
) -> Grammar {
    let nonterminals = given_nonterminals
        .split_whitespace()
        .map(|nt| NonTerminal(nt.into()))
        .collect::<HashSet<_>>();
    let terminals = terminals
        .split_whitespace()
        .map(|t| Terminal(t.into()))
        .collect();
    let mut productions = vec![];
    for production in given_productions {
        let lhs = production.0;
        for rhs in production.1.split('|').map(|s| s.trim()) {
            productions.push(Production {
                lhs: NonTerminal(lhs.to_string()),
                rhs: {
                    rhs.split_whitespace()
                        .map(|s| {
                            let s = s.to_string();
                            if nonterminals.contains(&NonTerminal(s.clone())) {
                                Symbol::NonTerminal(NonTerminal(s))
                            } else {
                                Symbol::Terminal(Terminal(s))
                            }
                        })
                        .collect()
                },
            })
        }
    }
    Grammar {
        nonterminals,
        terminals,
        productions,
        start: NonTerminal(start.into()),
    }
}
