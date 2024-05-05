use std::collections::{HashMap, HashSet};

use crate::{
    grammar::{Grammar, NonTerminal, Production, Symbol, Terminal},
    parse_tree::ParseTree,
};

type FirstSet<'a> = HashMap<&'a NonTerminal, HashSet<Option<&'a Terminal>>>;
type FollowSet<'a> = HashMap<&'a NonTerminal, HashSet<&'a Terminal>>;
type Table<'a> = HashMap<(&'a NonTerminal, &'a Terminal), Vec<&'a Production>>;

/// Returns first(`rhs`) - {ε} and if ε ∈ first(`rhs`)
fn first_rhs<'a>(rhs: &'a [Symbol], first: &FirstSet<'a>) -> (HashSet<&'a Terminal>, bool) {
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

fn create_first(grammar: &Grammar) -> FirstSet {
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

pub fn create_follow<'a>(grammar: &'a Grammar, first: &'a FirstSet) -> FollowSet<'a> {
    let mut follow = grammar
        .nonterminals()
        .iter()
        .zip(std::iter::repeat_with(HashSet::new))
        .collect::<HashMap<_, _>>();
    follow
        .get_mut(grammar.start())
        .unwrap()
        .insert(Symbol::eoim());
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

pub fn create_table<'a>(
    grammar: &'a Grammar,
    first: &'a FirstSet,
    follow: &'a FollowSet,
) -> Table<'a> {
    let mut table = HashMap::new();
    for nt in grammar.nonterminals() {
        for t in grammar
            .terminals()
            .iter()
            .chain(std::iter::once(Symbol::eoim()))
        {
            table.insert((nt, t), vec![]);
        }
    }
    for production in grammar.productions() {
        let lhs = production.lhs();
        let (terminals, nullable) = first_rhs(production.rhs(), first);

        for terminal in terminals.iter() {
            table.get_mut(&(lhs, terminal)).unwrap().push(production);
        }

        if nullable {
            for terminal in follow[lhs].iter() {
                table.get_mut(&(lhs, terminal)).unwrap().push(production);
            }
        }
    }
    table
}

pub fn parse<'a>(grammar: &'a Grammar, tokens: &[&str]) -> Result<Option<ParseTree<'a>>, ()> {
    let first = create_first(grammar);
    let follow = create_follow(grammar, &first);

    println!("First: ");
    for (k, v) in first.iter() {
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
        println!("{}: {}", k, v);
    }
    println!();
    println!("Follow: ");
    for (k, v) in follow.iter() {
        let v = v
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        println!("{}: {}", k, v);
    }

    println!();
    println!("Table: ");
    let table = create_table(grammar, &first, &follow);
    for (k, v) in table.iter() {
        if v.is_empty() {
            continue;
        }
        println!(
            "{} {}\t{}",
            k.0,
            k.1,
            v.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    for v in table.values() {
        if v.len() > 1 {
            return Err(());
        }
    }

    Ok(None)
}
