use std::collections::HashSet;

use parsing::{
    grammar::{Grammar, NonTerminal, Production, Symbol, Terminal},
    recursive_descent,
};

fn build_grammar(
    given_nonterminals: Vec<impl Into<String>>,
    terminals: Vec<impl Into<String>>,
    given_productions: Vec<(impl Into<String>, Vec<Vec<impl Into<String>>>)>,
    start: impl Into<String>,
) -> Grammar {
    let nonterminals = given_nonterminals
        .into_iter()
        .map(|nt| NonTerminal(nt.into()))
        .collect::<HashSet<_>>();
    let terminals = terminals.into_iter().map(|t| Terminal(t.into())).collect();
    let mut productions = vec![];
    for production in given_productions {
        let lhs = production.0.into();
        for rhs in production.1 {
            productions.push(Production {
                lhs: NonTerminal(lhs.clone()),
                rhs: {
                    rhs.into_iter()
                        .map(|s| {
                            let s = s.into();
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

fn main() {
    let grammar = build_grammar(
        ["E", "E'", "T", "T'", "F"].into(),
        ["+", "*", "(", ")", "ID", "$"].into(),
        vec![
            // ("E", vec![vec!["T", "E'", "$"]]),
            ("E", vec![vec!["T", "E'"]]),
            ("E'", vec![vec![], vec!["+", "T", "E'"]]),
            ("T", vec![vec!["F", "T'"]]),
            ("T'", vec![vec!["*", "F", "T'"], vec![]]),
            ("F", vec![vec!["(", "E", ")"], vec!["ID"]]),
        ],
        "E",
    );

    // let tokens = ["ID", "+", "ID"].map(ToString::to_string);
    let tokens = ["ID", "+", "ID", "*", "ID", "+", "ID"].map(ToString::to_string);
    // let tokens = ["ID", "*", "ID", "+", "ID"].map(ToString::to_string);
    // let tokens = ["ID"].map(ToString::to_string);

    let res = recursive_descent::parse(&grammar, &tokens);
    // println!("{:?}", res);
    if let Some(r) = res {
        println!("{}", r);
    }
}
