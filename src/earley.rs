// With credit to https://loup-vaillant.fr/tutorials/earley-parsing/recogniser

use std::{collections::HashMap, fmt, marker::PhantomData};

use crate::{
    grammar::{Grammar, Production, Symbol},
    parse_tree::ParseTree,
};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Complete;
#[derive(Debug, Clone, Copy, PartialEq)]
struct Incomplete;

#[derive(Debug, Clone, Copy, PartialEq)]
struct ItemBase<'a, C> {
    production: &'a Production,
    dot_idx: usize,
    _complete: PhantomData<C>,
}

impl<'a, C> fmt::Display for ItemBase<'a, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let before = self.production.rhs()[..self.dot_idx]
            .iter()
            .map(|s| match s {
                Symbol::Terminal(s) => s.0.as_str(),
                Symbol::NonTerminal(s) => s.0.as_str(),
            })
            .collect::<Vec<_>>()
            .join(" ");
        let after = self.production.rhs()[self.dot_idx..]
            .iter()
            .map(|s| match s {
                Symbol::Terminal(s) => s.0.as_str(),
                Symbol::NonTerminal(s) => s.0.as_str(),
            })
            .collect::<Vec<_>>()
            .join(" ");

        write!(
            f,
            "{} -> {}{}.{}{}",
            self.production.lhs().0,
            before,
            if before.is_empty() { "" } else { " " },
            if after.is_empty() { "" } else { " " },
            after,
        )
    }
}

impl<'a> ItemBase<'a, Incomplete> {
    pub fn next_symbol(&self) -> &'a Symbol {
        &self.production.rhs()[self.dot_idx]
    }

    pub fn to_next(mut self) -> Item<'a> {
        self.dot_idx += 1;
        if self.dot_idx < self.production.rhs().len() {
            Item::Incomplete(self)
        } else {
            Item::Complete(self.into())
        }
    }
}

impl<'a> From<ItemBase<'a, Incomplete>> for ItemBase<'a, Complete> {
    fn from(value: ItemBase<'a, Incomplete>) -> Self {
        Self {
            production: value.production,
            dot_idx: value.dot_idx,
            _complete: PhantomData,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Item<'a> {
    Incomplete(ItemBase<'a, Incomplete>),
    Complete(ItemBase<'a, Complete>),
}

impl<'a> Item<'a> {
    fn production(&self) -> &'a Production {
        match self {
            Item::Incomplete(i) => i.production,
            Item::Complete(i) => i.production,
        }
    }
}

impl<'a> fmt::Display for Item<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Item::Incomplete(item) => item.fmt(f),
            Item::Complete(item) => item.fmt(f),
        }
    }
}

impl<'a> Item<'a> {
    pub fn new(production: &'a Production) -> Self {
        if production.rhs().is_empty() {
            Self::Complete(ItemBase {
                production,
                dot_idx: 0,
                _complete: PhantomData,
            })
        } else {
            Self::Incomplete(ItemBase {
                production,
                dot_idx: 0,
                _complete: PhantomData,
            })
        }
    }
}

#[derive(Debug, Clone)]
enum HistoryValue {
    Scan {
        prev: (usize, usize),
    },
    Complete {
        prev: (usize, usize),
        // parent is really the child production
        parent: (usize, usize),
    },
}

/// Builds all possible parse trees for the item at `current`.
///
/// If the inputs are valid, then the tree is built up to the location of the dot
/// with the root being the nonterminal of the lhs of the production a `current`.
fn build_trees<'a>(
    states: &[Vec<(Item<'a>, usize)>],
    hist: &HashMap<(usize, usize), Vec<HistoryValue>>,
    current: (usize, usize),
) -> Vec<ParseTree<'a>> {
    let lhs = &states[current.0][current.1].0.production().lhs();

    // The row for parse tree children in built in reverse
    let mut bag = vec![(current, vec![])];
    let mut parse_trees = vec![];
    while let Some((current, mut row)) = bag.pop() {
        assert_eq!(lhs, &states[current.0][current.1].0.production().lhs());
        if let Some(hists) = hist.get(&current) {
            for history in hists.iter() {
                match *history {
                    HistoryValue::Scan { prev } => {
                        let Item::Incomplete(item) = &states[prev.0][prev.1].0 else {
                            panic!("Not an Incomplete");
                        };
                        let Symbol::Terminal(t) = item.next_symbol() else {
                            panic!("Not a terminal");
                        };

                        let mut row = row.clone();
                        row.push(ParseTree::Terminal(t));
                        bag.push((prev, row));
                    }
                    HistoryValue::Complete { prev, parent } => {
                        let trees = build_trees(states, hist, parent);
                        for tree in trees.into_iter() {
                            let mut row = row.clone();
                            row.push(tree);
                            bag.push((prev, row));
                        }
                    }
                }
            }
        } else {
            // Was a predict
            // Dot is at the start of the rhs so we have finished building the parse tree
            row.reverse();
            parse_trees.push(row);
        }
    }
    parse_trees
        .into_iter()
        .map(|row| ParseTree::NonTerminal(lhs, row))
        .collect()
}

pub fn parse<'a>(grammar: &'a Grammar, tokens: &[&str]) -> Vec<ParseTree<'a>> {
    let mut states = vec![vec![]; tokens.len() + 1];
    for production in grammar.productions_from(grammar.start()) {
        states[0].push((Item::new(production), 0));
    }
    let mut hist = HashMap::<_, Vec<_>>::new();

    for end in 0..states.len() {
        let (front, current, tail) = {
            let (left, tail) = states.split_at_mut(end + 1);
            let (front, current) = left.split_at_mut(end);
            (front, &mut current[0], tail)
        };

        let mut item_idx = 0;
        while let Some(&(item, start)) = current.get(item_idx) {
            // `end`: index in tokens of dot.
            // `start`: index in tokens of start symbol of production
            match item {
                Item::Incomplete(item) => match item.next_symbol() {
                    Symbol::Terminal(symbol) => {
                        // Scan
                        if let Some(&token) = tokens.get(end) {
                            if token == symbol.0 {
                                let next = &mut tail[0];
                                let entry = (item.to_next(), start);
                                // Only insert into state set if it's not already there.
                                // But either way we still need tò add the entry into the history.
                                let idx =
                                    next.iter().position(|x| x == &entry).unwrap_or_else(|| {
                                        next.push(entry);
                                        next.len() - 1
                                    });
                                hist.entry((end + 1, idx))
                                    .or_default()
                                    .push(HistoryValue::Scan {
                                        prev: (end, item_idx),
                                    });
                            }
                        }
                    }

                    Symbol::NonTerminal(symbol) => {
                        // Predict
                        for production in grammar.productions_from(symbol) {
                            let value = (Item::new(production), end);
                            if !current.contains(&value) {
                                current.push(value);
                            }
                        }
                    }
                },
                Item::Complete(item) => {
                    // Complete

                    let symbol = item.production.lhs();

                    // Now search for possible parents.
                    // The end of parent == start of current item
                    // (end being the location of dot)
                    let parent_end_set = if start == end {
                        &*current
                    } else {
                        &front[start]
                    };
                    let mut to_add = vec![];
                    for (parent_idx, &(parent_item, parent_start)) in
                        parent_end_set.iter().enumerate()
                    {
                        if let Item::Incomplete(parent_item) = parent_item {
                            if let Symbol::NonTerminal(parent_symbol) = parent_item.next_symbol() {
                                if parent_symbol == symbol {
                                    // The predict step in parent created current item.
                                    // Now, for the parent, move the dot over the next symbol
                                    // (must have been a nonterminal the matches the lhs of the
                                    // current production) and add it to the state set.
                                    //
                                    // For the history of the new item (parent with dot moved), we push on:
                                    // [(prev item of parent - with the dot one place back: ie. the original parent, (start == parent_end)
                                    //   pointer to completed item of the just completed nonterminal in parent: ie. the current item)]
                                    //
                                    to_add.push((
                                        (parent_item.to_next(), parent_start),
                                        // ((start, parent_idx), (end, item_idx)),
                                        HistoryValue::Complete {
                                            prev: (start, parent_idx),
                                            parent: (end, item_idx),
                                        },
                                    ));
                                }
                            }
                        }
                    }
                    for (entry, history_value) in to_add {
                        // Only insert into state set if it's not already there.
                        // But either way we still need tò add the entry into the history.
                        let idx = current.iter().position(|x| x == &entry).unwrap_or_else(|| {
                            current.push(entry);
                            current.len() - 1
                        });
                        hist.entry((end, idx)).or_default().push(history_value);
                    }
                }
            }
            item_idx += 1;
        }
    }

    let mut collect = hist.iter().collect::<Vec<_>>();
    collect.sort_by_key(|(&k, _)| k);
    for (&current, hists) in collect {
        for history in hists.iter() {
            let (prev, parent) = match *history {
                HistoryValue::Scan { prev } => (prev, None),
                HistoryValue::Complete { prev, parent } => (prev, Some(parent)),
            };
            println!(
                "{:?}\t{}\t{}\t{}",
                (current, prev, parent.unwrap_or((69, 69))),
                states[current.0][current.1].0,
                states[prev.0][prev.1].0,
                if let Some(parent) = parent {
                    states[parent.0][parent.1].0.to_string()
                } else {
                    "".to_string()
                },
            );
        }
    }

    states
        .last()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, &entry)| {
            if let (Item::Complete(item), 0) = entry {
                item.production.lhs() == grammar.start()
            } else {
                false
            }
        })
        .flat_map(|(idx, _)| build_trees(&states, &hist, (states.len() - 1, idx)))
        .collect()
}
