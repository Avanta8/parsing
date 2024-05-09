use std::{fmt, marker::PhantomData};

use crate::grammar::{NonTerminal, Production, Symbol, Terminal};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Complete;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Incomplete;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemBase<'a, C> {
    production: &'a Production,
    dot_idx: usize,
    _complete: PhantomData<C>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Item<'a> {
    Incomplete(ItemBase<'a, Incomplete>),
    Complete(ItemBase<'a, Complete>),
}

impl<'a, C> ItemBase<'a, C> {
    pub fn production(&self) -> &'a Production {
        self.production
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

    pub fn production(&self) -> &'a Production {
        match self {
            Item::Incomplete(i) => i.production(),
            Item::Complete(i) => i.production(),
        }
    }
}

impl<'a> ItemBase<'a, Incomplete> {
    pub fn next_symbol(&self) -> &'a Symbol<Terminal, NonTerminal> {
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

impl<'a, C> fmt::Display for ItemBase<'a, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let before = self.production.rhs()[..self.dot_idx]
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ");
        let after = self.production.rhs()[self.dot_idx..]
            .iter()
            .map(ToString::to_string)
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

impl<'a> fmt::Display for Item<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Item::Incomplete(item) => item.fmt(f),
            Item::Complete(item) => item.fmt(f),
        }
    }
}
