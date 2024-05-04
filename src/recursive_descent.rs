use std::collections::VecDeque;

use crate::{
    ast::{Ast, AstNode},
    grammar::{Grammar, NonTerminal, Symbol, Terminal},
};

#[derive(Debug, Clone)]
enum Elem<'a> {
    Terminal(&'a Terminal),
    Unexpanded(&'a NonTerminal),
    Expanded(&'a NonTerminal, usize), // usize is arena_idx of child tree
}

enum Step<'a> {
    Terminal(Tree<'a>),
    NonTerminal(Vec<Tree<'a>>),
}

#[derive(Debug, Clone)]
struct Tree<'a> {
    grammar: &'a Grammar,
    arena: Vec<Vec<Elem<'a>>>,
    // Invariant: position must be valid
    position: (usize, usize),    // (arena_idx, rhs_idx)
    parent: Vec<(usize, usize)>, // [position]
}

impl<'a> Tree<'a> {
    fn new(grammar: &'a Grammar) -> Self {
        Self {
            grammar,
            arena: vec![vec![Elem::Unexpanded(&grammar.start)]],
            position: (0, 0),
            parent: vec![(69, 69)], // HACK: Change parent of root to something unique
        }
    }

    fn is_complete(&self) -> bool {
        self.position == (69, 69)
    }

    fn normalize(&mut self) {
        while self.position.1 >= self.arena[self.position.0].len() {
            self.position = self.parent[self.position.0];
            if self.position == (69, 69) {
                break;
            }
            self.position.1 += 1;
        }
    }

    fn step(mut self, token: Option<&str>) -> Step<'a> {
        match self.arena[self.position.0][self.position.1] {
            Elem::Terminal(t) => {
                if Some(t.0.as_str()) != token {
                    // HACK:
                    return Step::NonTerminal([].into());
                }
                self.position.1 += 1;
                self.normalize();
                Step::Terminal(self)
            }
            Elem::Unexpanded(nt) => {
                let mut new_trees = vec![];
                for production in self.grammar.productions_from(nt) {
                    let mut new_tree = self.clone();
                    let subtree = production
                        .rhs
                        .iter()
                        .map(|s| match s {
                            Symbol::Terminal(t) => Elem::Terminal(t),
                            Symbol::NonTerminal(nt) => Elem::Unexpanded(nt),
                        })
                        .collect();
                    new_tree.arena[self.position.0][self.position.1] =
                        Elem::Expanded(nt, self.arena.len());
                    new_tree.position = (self.arena.len(), 0);
                    new_tree.arena.push(subtree);
                    new_tree.parent.push(self.position);

                    new_tree.normalize();
                    new_trees.push(new_tree);
                }
                Step::NonTerminal(new_trees)
            }
            Elem::Expanded(_, _) => unreachable!("Tree stepped into already expanded nonterminal"),
        }
    }

    fn idx_to_astnodes(&self, arena_idx: usize) -> Vec<AstNode<'a>> {
        let elems = &self.arena[arena_idx];
        let children = elems
            .iter()
            .map(|elem| match elem {
                Elem::Terminal(t) => AstNode::Terminal(t),
                Elem::Unexpanded(_) => unreachable!("Trying to convert unexpanded into AST"),
                &Elem::Expanded(nt, arena_idx) => {
                    AstNode::NonTerminal(Ast::new(nt, self.idx_to_astnodes(arena_idx)))
                }
            })
            .collect::<Vec<_>>();
        children
    }

    fn to_ast(&self) -> Ast<'a> {
        let AstNode::NonTerminal(start) = self.idx_to_astnodes(0).swap_remove(0) else {
            panic!()
        };
        start
    }
}

pub fn parse<'a>(grammar: &'a Grammar, tokens: &'a [String]) -> Option<Ast<'a>> {
    let mut bag = VecDeque::from([(Tree::new(grammar), 0)]);
    while let Some((tree, idx)) = bag.pop_back() {
        dbg!(idx);
        // dbg!((&tree, idx));
        if idx == tokens.len() && tree.is_complete() {
            return Some(tree.to_ast());
        } else if tree.is_complete() {
            continue;
        }

        match tree.step(tokens.get(idx).map(|x| x.as_str())) {
            Step::Terminal(new_tree) => bag.push_front((new_tree, idx + 1)),
            Step::NonTerminal(new_trees) => {
                for new_tree in new_trees {
                    bag.push_front((new_tree, idx));
                }
            }
        }
    }
    None
}
