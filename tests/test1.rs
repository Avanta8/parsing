use parsing::{
    earley,
    grammar::{build_grammar, Grammar},
    ll1, recursive_descent,
};

struct Setup {
    grammar: Grammar,
    tokens: Vec<&'static str>,
    ans: &'static str,
}

impl Setup {
    fn new() -> Self {
        let grammar = build_grammar(
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
        );
        let string = "w + x * ( y + z ) * w + y * x";
        let tokens = string.split_whitespace().collect::<Vec<_>>();
        let ans = "E\tT\tF\tID\tw\n\t\tT'\n\tE'\t+\n\t\tT\tF\tID\tx\n\t\t\tT'\t*\n\t\t\t\tF\t(\n\t\t\t\t\tE\tT\tF\tID\ty\n\t\t\t\t\t\t\tT'\n\t\t\t\t\t\tE'\t+\n\t\t\t\t\t\t\tT\tF\tID\tz\n\t\t\t\t\t\t\t\tT'\n\t\t\t\t\t\t\tE'\n\t\t\t\t\t)\n\t\t\t\tT'\t*\n\t\t\t\t\tF\tID\tw\n\t\t\t\t\tT'\n\t\tE'\t+\n\t\t\tT\tF\tID\ty\n\t\t\t\tT'\t*\n\t\t\t\t\tF\tID\tx\n\t\t\t\t\tT'\n\t\t\tE'";
        Self {
            grammar,
            tokens,
            ans,
        }
    }
}

#[test]
fn recurive_descent() {
    let setup = Setup::new();
    let ans = recursive_descent::parse(&setup.grammar, &setup.tokens).unwrap();
    assert_eq!(ans.to_string(), setup.ans);
}

#[test]
fn earley() {
    let setup = Setup::new();
    let ans = earley::parse(&setup.grammar, &setup.tokens);
    assert_eq!(ans.len(), 1);
    assert_eq!(ans[0].to_string(), setup.ans);
}

#[test]
fn ll1() {
    let setup = Setup::new();
    let ll1::ParseResult::Parse(ans) = ll1::parse(&setup.grammar, &setup.tokens) else {
        panic!();
    };
    assert_eq!(ans.to_string(), setup.ans);
}
