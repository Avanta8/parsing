use parsing::{
    grammar::{build_grammar, Grammar},
    ll1,
};

fn run(grammar: &Grammar, string: &str) {
    let tokens = string.split_whitespace().collect::<Vec<_>>();
    let res = ll1::parse(grammar, &tokens);
    match res {
        ll1::ParseResult::Conflict => println!("conflict"),
        ll1::ParseResult::NoParse => println!("no parser"),
        ll1::ParseResult::Parse(res) => println!("{}\n", res),
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
            "E ID",
            "+ * ( ) w x y z",
            vec![("E", "E + E | E * E | ( E ) | ID"), ("ID", "w | x | y | z")],
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
        build_grammar("S", "b", vec![("S", "S S | b")], "S"),
    ];
    for grammar in grammars[0..3].iter() {
        let string = "w + x * ( y + z ) * w + y * x";
        run(grammar, string);
    }
    for grammar in grammars[3..4].iter() {
        let string = "they can fish";
        run(grammar, string);
    }
    for grammar in grammars[4..5].iter() {
        let string = "b b b";
        run(grammar, string);
    }
}
