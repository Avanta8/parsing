use std::collections::{HashMap, HashSet};

use crate::grammar::{Grammar, NonTerminal, Symbol, Terminal};

pub type FirstSet<'a> = HashMap<&'a NonTerminal, HashSet<Option<&'a Terminal>>>;
pub type FollowSet<'a> = HashMap<&'a NonTerminal, HashSet<&'a Terminal>>;

/// Returns first(`rhs`) - {ε} and if ε ∈ first(`rhs`)
pub fn first_rhs<'a>(
    rhs: &'a [Symbol<Terminal, NonTerminal>],
    first: &FirstSet<'a>,
) -> (HashSet<&'a Terminal>, bool) {
    let mut set = HashSet::new();
    let mut nullable = true;
    for symbol in rhs.iter() {
        if !nullable {
            break;
        }
        nullable = false;
        match symbol {
            Symbol::Terminal(t) => {
                set.insert(t);
            }
            Symbol::NonTerminal(nt) => {
                let elements = first[nt].iter().copied().collect::<Vec<_>>();
                for element in elements {
                    if let Some(element) = element {
                        set.insert(element);
                    } else {
                        nullable = true;
                    }
                }
            }
        }
    }
    (set, nullable)
}

pub fn create_first(grammar: &Grammar) -> FirstSet {
    // None represents epsilon
    //
    // first[nt] contains None only if nt is nullable.
    // Relationship should be iff by the end of the fixed point iterations.
    // (But I should probably prove this)
    let mut first = grammar
        .nonterminals()
        .iter()
        .zip(std::iter::repeat_with(HashSet::new))
        .collect::<HashMap<_, _>>();

    let mut changed = true;
    while changed {
        changed = false;
        for production in grammar.productions() {
            let (terminals, nullable) = first_rhs(production.rhs(), &first);
            let set = first.get_mut(production.lhs()).unwrap();
            for terminal in terminals {
                changed |= set.insert(Some(terminal));
            }
            if nullable {
                changed |= set.insert(None);
            }
        }
    }

    first
}

pub fn create_follow<'a>(grammar: &'a Grammar, first: &FirstSet<'a>) -> FollowSet<'a> {
    let mut follow = grammar
        .nonterminals()
        .iter()
        .zip(std::iter::repeat_with(HashSet::new))
        .collect::<HashMap<_, _>>();
    follow
        .get_mut(grammar.start())
        .unwrap()
        .insert(Terminal::eoim());
    let mut changed = true;
    while changed {
        changed = false;
        for production in grammar.productions() {
            let lhs = production.lhs();
            let rhs = production.rhs();

            for (i, current) in rhs.iter().enumerate() {
                let Symbol::NonTerminal(current) = current else {
                    continue;
                };
                let (mut terminals, nullable) = first_rhs(&rhs[i + 1..], first);

                if nullable {
                    // Either the remaining rhs is nullable, or current is the
                    // last symbol in the rhs
                    terminals.extend(follow[lhs].iter());
                }

                let set = follow.get_mut(current).unwrap();
                for terminal in terminals {
                    changed |= set.insert(terminal);
                }
            }
        }
    }
    follow
}

pub fn first_to_string(first: &FirstSet) -> String {
    first
        .iter()
        .map(|(k, v)| {
            let v = v
                .iter()
                .map(|s| {
                    if let Some(s) = s {
                        s.to_string()
                    } else {
                        "ε".to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}: {}", k, v)
        })
        .chain(std::iter::once("".to_string()))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn follow_to_string(follow: &FollowSet) -> String {
    follow
        .iter()
        .map(|(k, v)| {
            let v = v
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}: {}", k, v)
        })
        .chain(std::iter::once("".to_string()))
        .collect::<Vec<_>>()
        .join("\n")
}
