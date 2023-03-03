// #![feature(once_cell)]
#![feature(generic_const_exprs)]

// mod net;
mod inet;

fn main() {
    inet_main()
}

use crate::inet::{
    cell::CellPtr,
    equation::{EquationBuilder, EquationPtr},
    net::Net,
    rule::RuleBook,
    runtime::Runtime,
    symbol::SymbolBook,
    Polarity,
};

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

    // -- def sub1 := init
    // #eval eval "sub2-1" [ ⟨Sub (fvar 0) Two, One⟩ ]
    // #eval eval "sub3-2" [ ⟨Sub (fvar 0) (natToTerm 3), Two⟩ ]
    // #eval eval "sub3-0" [ ⟨Sub (fvar 0) (natToTerm 3), Z⟩ ]
    // #eval eval "sub1-2" [ ⟨Sub (fvar 0) One, Two⟩ ]
    // net.equations(|b| {
    //     let result = b.fvar();

    //     let a = b.zero();
    // });

    println!("{}", net);

    let mut runtime = Runtime::new(&symbols, &rules);

    println!();

    let net = runtime.eval(net);

    println!("{}", net);
    println!();
}
