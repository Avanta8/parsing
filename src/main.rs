use std::collections::HashSet;

use parsing::{
    earley,
    grammar::{Grammar, NonTerminal, Production, Symbol, Terminal},
    recursive_descent,
};

fn build_grammar(
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

fn run(grammar: &Grammar, s: &str) {
    let tokens = s.split_whitespace().collect::<Vec<_>>();
    // let res = recursive_descent::parse(grammar, &tokens);
    let res = earley::parse(grammar, &tokens);
    println!("finished");
    if let Some(r) = res {
        // println!("{}", r);
    }
}

fn main() {
    let grammars = [
        build_grammar(
            "E E' T T' F ID",
            "+ * ( ) x y z w",
            vec![
                ("E", "T E'"),
                ("E'", "+ T E' | "),
                ("T", "F T'"),
                ("T'", "* F T' | "),
                ("F", "( E ) | ID"),
                ("ID", "w | x | y | z"),
            ],
            "E",
        ),
        build_grammar(
            "E T F ID",
            "+ * ( ) w x y z",
            vec![
                ("E", "E + T | T"),
                ("T", "T * F | F"),
                ("F", "( E ) | ID"),
                ("ID", "w | x | y | z"),
            ],
            "E",
        ),
        build_grammar(
            "S NP VP PP N V P",
            "can fish in rivers they",
            vec![
                ("S", "NP VP"),
                ("NP", "N PP | N"),
                ("PP", "P NP"),
                ("VP", "VP PP | V VP | V NP | V"),
                ("N", "can | they | fish | rivers"),
                ("P", "in"),
                ("V", "can | fish"),
            ],
            "S",
        ),
    ];
    for grammar in grammars[0..2].iter() {
        let tokens = "w + x * ( y + z ) * w";
        run(grammar, tokens);
    }
    for grammar in grammars[2..3].iter() {
        let tokens = "they can fish";
        run(grammar, tokens);
    }
}
