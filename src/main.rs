// #![feature(once_cell)]
#![feature(generic_const_exprs)]

// mod net;
mod inet;

fn main() {
    inet_main()
}

use crate::inet::{net::Net, rule::RuleBook, runtime::Runtime, symbol::SymbolBook};

mod examples;

pub fn inet_main() {
    // symbols
    let mut symbols = SymbolBook::new();
    symbols.declare_nat_symbols();
    symbols.declare_arith_symbols();
    symbols.declare_combinator_symbols();

    // // Fib
    // let fib_sym = symbols.declare1("fib", Polarity::Neg, Polarity::Pos);
    // let fib0_sym = symbols.declare1("fib₀", Polarity::Neg, Polarity::Pos);

    println!("{}", symbols);

    // rules
    let mut rules = RuleBook::new(&symbols);

    // Arith rules
    rules.arith_rules();
    rules.define_combinator_rules();

    println!("{}", rules);
    // println!("{:?}", rules);

    println!();

    // net
    let mut net = Net::new(&symbols);

    // 1 + 0 = 1
    net.equations(|b| {
        let one = b.one();
        let r_fvar = b.fvar();
        let adder = b.adder(r_fvar.into(), one.into());

        let zero = b.zero();
        // ATTN: cannot "share" cells!! TODO how to avoid this?
        b.add(zero, adder);
    });

    // 1 + 2 = 3
    net.equations(|b| {
        let one = b.one();
        let r_fvar = b.fvar();
        let adder = b.adder(r_fvar.into(), one.into());

        // ATTN: cannot "share" cells!! TODO how to avoid this?
        let two = b.two();
        b.add(two, adder);
    });

    // 2 - 1 = 1
    net.equations(|b| {
        let result = b.fvar();
        let one = b.one();
        let subtractor = b.subtractor(result.into(), one.into());

        let two = b.two();
        b.subtract(two, subtractor);
    });

    // 3 - 2 = 1
    net.equations(|b| {
        let result = b.fvar();
        let two = b.two();
        let three = b.succ(two.into());
        let subtractor = b.subtractor(result.into(), three.into());

        let two = b.two();
        b.subtract(two, subtractor);
    });

    // 3 - 0 = 0
    net.equations(|b| {
        let result = b.fvar();
        let two = b.two();
        let three = b.succ(two.into());
        let subtractor = b.subtractor(result.into(), three.into());

        let zero = b.zero();
        b.subtract(zero, subtractor);
    });

    // 1 - 2 = 0
    net.equations(|b| {
        let result = b.fvar();
        let one = b.two();
        let subtractor = b.subtractor(result.into(), one.into());

        let two = b.two();
        b.subtract(two, subtractor);
    });

    println!("{}", net);

    let mut runtime = Runtime::new(&symbols, &rules);

    println!();

    let net = runtime.eval(net);

    println!("{}", net);
    println!();
}
