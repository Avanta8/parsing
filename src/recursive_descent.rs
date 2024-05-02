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
    Terminal(TreeResult<'a>),
    NonTerminal(Vec<TreeResult<'a>>),
}

enum TreeResult<'a> {
    Incomplete(Tree<'a, Incomplete>),
    Complete(Tree<'a, Complete>),
}

impl<'a> TreeResult<'a> {
    fn new(grammar: &'a Grammar) -> Self {
        Tree::<Incomplete> {
            grammar,
            arena: vec![vec![Elem::Unexpanded(&grammar.start)]],
            position: (0, 0),
            parent: vec![None],
            _complete: std::marker::PhantomData,
        }
        .normalize()
    }
}

#[derive(Clone)]
struct Incomplete;
#[derive(Clone)]
struct Complete;

#[derive(Debug, Clone)]
struct Tree<'a, C> {
    grammar: &'a Grammar,
    arena: Vec<Vec<Elem<'a>>>,
    // Invariant: position must be valid
    position: (usize, usize),            // (arena_idx, rhs_idx)
    parent: Vec<Option<(usize, usize)>>, // [position]. Parent of root is None
    _complete: std::marker::PhantomData<C>,
}

impl<'a> From<Tree<'a, Incomplete>> for Tree<'a, Complete> {
    fn from(value: Tree<'a, Incomplete>) -> Self {
        Self {
            grammar: value.grammar,
            arena: value.arena,
            position: value.position,
            parent: value.parent,
            _complete: std::marker::PhantomData,
        }
    }
}

impl<'a> Tree<'a, Incomplete> {
    fn normalize(mut self) -> TreeResult<'a> {
        while self.position.1 >= self.arena[self.position.0].len() {
            let Some(position) = self.parent[self.position.0] else {
                return TreeResult::Complete(self.into());
            };
            self.position = position;
            self.position.1 += 1;
        }
        TreeResult::Incomplete(self)
    }

    fn step(mut self, token: Option<&str>) -> Step<'a> {
        match self.arena[self.position.0][self.position.1] {
            Elem::Terminal(t) => {
                if Some(t.0.as_str()) != token {
                    // HACK:
                    return Step::NonTerminal([].into());
                }
                self.position.1 += 1;
                Step::Terminal(self.normalize())
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
                    new_tree.parent.push(Some(self.position));

                    new_trees.push(new_tree.normalize());
                }
                Step::NonTerminal(new_trees)
            }
            Elem::Expanded(_, _) => unreachable!("Tree stepped into already expanded nonterminal"),
        }
    }
}

impl<'a> Tree<'a, Complete> {
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
            panic!("Root was not a NonTerminal")
        };
        start
    }
}

pub fn parse<'a>(grammar: &'a Grammar, tokens: &'a [String]) -> Option<Ast<'a>> {
    // let mut bag = VecDeque::from([(Tree::new(grammar), 0)]);
    let mut bag = VecDeque::from([(TreeResult::new(grammar), 0)]);
    while let Some((tree_result, idx)) = bag.pop_back() {
        match tree_result {
            TreeResult::Incomplete(tree) => {
                dbg!(idx);
                // dbg!((&tree, idx));

                match tree.step(tokens.get(idx).map(|x| x.as_str())) {
                    Step::Terminal(tree_result) => bag.push_front((tree_result, idx + 1)),
                    Step::NonTerminal(tree_results) => {
                        for tree_result in tree_results {
                            bag.push_front((tree_result, idx));
                        }
                    }
                }
            }

            TreeResult::Complete(tree) => {
                if idx == tokens.len() {
                    return Some(tree.to_ast());
                }
            }
        }
    }
    None
}
