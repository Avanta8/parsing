use std::{collections::HashSet, fmt, sync::OnceLock};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Terminal(pub String);

impl fmt::Display for Terminal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NonTerminal(pub String);

impl fmt::Display for NonTerminal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Symbol {
    Terminal(Terminal),
    NonTerminal(NonTerminal),
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Symbol::Terminal(s) => s.fmt(f),
            Symbol::NonTerminal(s) => s.fmt(f),
        }
    }
}

impl Symbol {
    /// End-of-input marker
    pub fn eoim() -> &'static Terminal {
        static T: OnceLock<Terminal> = OnceLock::new();
        T.get_or_init(|| Terminal("$".to_string()))
    }
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

impl fmt::Display for Production {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} -> {}",
            self.lhs(),
            self.rhs
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        )
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
