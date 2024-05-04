// With credit to https://loup-vaillant.fr/tutorials/earley-parsing/recogniser

use std::{collections::HashMap, fmt, marker::PhantomData};

use crate::grammar::{Grammar, Production, Symbol};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Complete;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Incomplete;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Item<'a, C> {
    production: &'a Production,
    dot_idx: usize,
    _complete: PhantomData<C>,
}

impl<'a, C> fmt::Display for Item<'a, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} -> {} . {}",
            self.production.lhs.0,
            self.production.rhs[..self.dot_idx]
                .iter()
                .map(|s| match s {
                    Symbol::Terminal(s) => s.0.as_str(),
                    Symbol::NonTerminal(s) => s.0.as_str(),
                })
                .collect::<Vec<_>>()
                .join(" "),
            self.production.rhs[self.dot_idx..]
                .iter()
                .map(|s| match s {
                    Symbol::Terminal(s) => s.0.as_str(),
                    Symbol::NonTerminal(s) => s.0.as_str(),
                })
                .collect::<Vec<_>>()
                .join(" "),
        )
    }
}

impl<'a> Item<'a, Incomplete> {
    pub fn next_symbol(&self) -> &Symbol {
        &self.production.rhs[self.dot_idx]
    }

    pub fn to_next(mut self) -> ItemEnum<'a> {
        self.dot_idx += 1;
        if self.dot_idx < self.production.rhs.len() {
            ItemEnum::Incomplete(self)
        } else {
            ItemEnum::Complete(self.into())
        }
    }
}

impl<'a> From<Item<'a, Incomplete>> for Item<'a, Complete> {
    fn from(value: Item<'a, Incomplete>) -> Self {
        Self {
            production: value.production,
            dot_idx: value.dot_idx,
            _complete: PhantomData,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ItemEnum<'a> {
    Incomplete(Item<'a, Incomplete>),
    Complete(Item<'a, Complete>),
}

impl<'a> fmt::Display for ItemEnum<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemEnum::Incomplete(item) => item.fmt(f),
            ItemEnum::Complete(item) => item.fmt(f),
        }
    }
}

impl<'a> ItemEnum<'a> {
    pub fn new(production: &'a Production) -> Self {
        if production.rhs.is_empty() {
            Self::Complete(Item::<Complete> {
                production,
                dot_idx: 0,
                _complete: PhantomData,
            })
        } else {
            Self::Incomplete(Item::<Incomplete> {
                production,
                dot_idx: 0,
                _complete: PhantomData,
            })
        }
    }
}

pub fn parse(grammar: &Grammar, tokens: &[&str]) -> Option<()> {
    let mut states = vec![vec![]; tokens.len() + 1];
    for production in grammar.productions_from(&grammar.start) {
        states[0].push((ItemEnum::new(production), 0));
    }
    let mut hist = HashMap::<_, Vec<_>>::new();
    for end in 0..states.len() {
        // HACK:
        let mut t = vec![];
        let taddr = &t as *const _;
        let (front, current, next) = {
            let (left, right) = states.split_at_mut(end + 1);
            let (front, current) = left.split_at_mut(end);
            (
                front,
                current.first_mut().unwrap(),
                right.first_mut().unwrap_or(&mut t),
            )
        };

        let mut item_idx = 0;
        while let Some(&(ref item, start)) = current.get(item_idx) {
            // println!("end: {end}, item_idx: {item_idx}");
            match item {
                ItemEnum::Incomplete(item) => match item.next_symbol() {
                    Symbol::Terminal(symbol) => {
                        // Scan
                        // TODO: Better end checking.
                        if end < tokens.len() && tokens[end] == symbol.0 {
                            let value = (item.clone().to_next(), start);
                            assert!(!std::ptr::eq(next, taddr));
                            if !next.contains(&value) {
                                next.push(value);
                            }
                        } else if end >= tokens.len() {
                            assert_eq!(end, tokens.len());
                            assert!(std::ptr::eq(next, taddr));
                        }
                    }

                    Symbol::NonTerminal(symbol) => {
                        // Predict
                        for production in grammar.productions_from(symbol) {
                            let value = (ItemEnum::new(production), end);
                            if !current.contains(&value) {
                                current.push(value);
                            }
                        }
                    }
                },
                ItemEnum::Complete(item) => {
                    // Complete

                    // TODO: Don't use clone
                    let symbol = Symbol::NonTerminal(item.production.lhs.clone());

                    #[allow(clippy::comparison_chain)]
                    if start == end {
                        // NOTE: This must be an epsilon production
                        // FIXME: Add to parents.
                        // Perhaps merge implementation with the general case.
                        assert!(item.production.rhs.is_empty());
                        let mut to_push = vec![];
                        for &(ref parent_item, parent_start) in current.iter() {
                            if let ItemEnum::Incomplete(parent_item) = parent_item {
                                if parent_item.next_symbol() == &symbol {
                                    to_push.push((parent_item.clone().to_next(), parent_start));
                                }
                            }
                        }
                        current.extend(to_push);
                    } else if start > end {
                        panic!("start > end");
                    } else {
                        for (parent_idx, &(ref parent_item, parent_start)) in
                            front[start].iter().enumerate()
                        {
                            if let ItemEnum::Incomplete(parent_item) = parent_item {
                                if parent_item.next_symbol() == &symbol {
                                    let value = (parent_item.clone().to_next(), parent_start);
                                    match current.iter().position(|x| x == &value) {
                                        Some(current_idx) => {
                                            hist.get_mut(&(end, current_idx))
                                                .unwrap()
                                                .push(((start, parent_idx), (end, item_idx)));
                                        }
                                        None => {
                                            hist.insert(
                                                (end, current.len()),
                                                vec![((start, parent_idx), (end, item_idx))],
                                            );
                                            current.push(value);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            item_idx += 1;
        }
    }
    for row in states[states.len() - 1].iter() {
        if matches!(row, (ItemEnum::Complete(_), 0)) {
            println!("{:?}", row);
        }
    }

    let mut collect = hist.into_iter().collect::<Vec<_>>();
    collect.sort();
    for (current, parents) in collect {
        for (prev, parent) in parents {
            println!(
                "{:?}\t{}\t{}",
                (current, prev, parent),
                states[current.0][current.1].0,
                states[parent.0][parent.1].0,
            );
        }
    }

    None
}