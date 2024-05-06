use std::collections::HashMap;

use crate::{
    first_follow::{create_first, create_follow, first_rhs, FirstSet, FollowSet},
    grammar::{Grammar, NonTerminal, Production, Symbol, Terminal},
    parse_tree::ParseTree,
};

pub type LL1Table<'a> = HashMap<(&'a NonTerminal, &'a Terminal), Vec<&'a Production>>;

#[derive(Debug, Clone)]
enum Tree<'a> {
    Terminal(&'a Terminal),
    Nonterminal(&'a NonTerminal, usize),
}

#[derive(Debug, Clone)]
pub enum ParseResult<'a> {
    Conflict,
    NoParse,
    Parse(ParseTree<'a>),
}

pub fn create_table<'a>(
    grammar: &'a Grammar,
    first: &FirstSet<'a>,
    follow: &FollowSet<'a>,
) -> LL1Table<'a> {
    let mut table = HashMap::new();
    for nt in grammar.nonterminals() {
        for t in grammar
            .terminals()
            .iter()
            .chain(std::iter::once(Terminal::eoim()))
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

pub fn table_to_string(table: &LL1Table) -> String {
    table
        .iter()
        .filter_map(|(k, v)| {
            if v.is_empty() {
                None
            } else {
                Some(format!(
                    "{} {}\t{}",
                    k.0,
                    k.1,
                    v.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            }
        })
        .chain(std::iter::once("".to_string()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn gen_parse_tree<'a>(trees: &[Vec<Tree<'a>>], current: &Tree<'a>) -> ParseTree<'a> {
    match *current {
        Tree::Terminal(t) => ParseTree::Terminal(t),
        Tree::Nonterminal(nt, children_idx) => ParseTree::NonTerminal(
            nt,
            trees[children_idx]
                .iter()
                .map(|child| gen_parse_tree(trees, child))
                .collect(),
        ),
    }
}

pub fn parse_with_table<'a>(
    grammar: &'a Grammar,
    tokens: &[&str],
    table: &LL1Table<'a>,
) -> ParseResult<'a> {
    let start = Symbol::NonTerminal(grammar.start());
    let eoim = Symbol::<&Terminal, &NonTerminal>::eoim();
    let mut stack = vec![(eoim.clone(), 0), (start, 0)];

    let mut trees = vec![vec![]];

    let mut idx = 0;

    while let Some((top, parent)) = stack.pop() {
        if top == eoim {
            break;
        }
        match top {
            Symbol::Terminal(top) => {
                if top.0 == tokens[idx] {
                    idx += 1;
                    // Add the terminal into the subtree of the parent.
                    trees[parent].push(Tree::Terminal(top));
                } else {
                    // fail to parse
                    return ParseResult::NoParse;
                }
            }
            Symbol::NonTerminal(top) => {
                // HACK:
                // Set the token to the end-of-input marker if we have reached the end of input.
                assert!(idx <= tokens.len());
                let token = Terminal(
                    tokens
                        .get(idx)
                        .copied()
                        .unwrap_or_else(|| Terminal::eoim().0.as_str())
                        .to_string(),
                );
                let entry = &table[&(top, &token)];
                assert!(entry.len() < 2);
                if let Some(production) = entry.first() {
                    assert_eq!(top, production.lhs());

                    // Add the subtree for this production.
                    trees.push(vec![]);

                    let value = trees.len() - 1;
                    // Add the subtree into the children of the parent.
                    trees[parent].push(Tree::Nonterminal(top, value));

                    // Push the rhs onto the stack in reverse.
                    stack.extend(
                        production
                            .rhs()
                            .iter()
                            .map(Symbol::as_ref)
                            .rev()
                            .zip(std::iter::repeat(value)),
                    );
                } else {
                    // fail to parse
                    return ParseResult::NoParse;
                }
            }
        }
    }

    assert_eq!(trees[0].len(), 1);
    ParseResult::Parse(gen_parse_tree(&trees, &trees[0][0]))
}

pub fn parse<'a>(grammar: &'a Grammar, tokens: &[&str]) -> ParseResult<'a> {
    let first = create_first(grammar);
    let follow = create_follow(grammar, &first);
    let table = create_table(grammar, &first, &follow);

    // println!("First:\n{}", first_to_string(&first));
    // println!("Follow:\n{}", follow_to_string(&follow));
    // println!("Table:\n{}", table_to_string(&table));

    for v in table.values() {
        if v.len() > 1 {
            return ParseResult::Conflict;
        }
    }

    parse_with_table(grammar, tokens, &table)
}
