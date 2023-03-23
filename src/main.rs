// #![feature(once_cell)]
#![feature(generic_const_exprs)]
#![feature(const_alloc_layout)]
#![feature(thread_local)]
// mod net;
mod inet;

fn main() {
    inet_main()
}

use rayon::Scope;

use crate::inet::{net::Net, rule::RuleSet, runtime::Runtime, symbol::SymbolBook};

mod examples;

pub fn inet_main() {
    // symbols
    let mut symbols = SymbolBook::new();
    symbols.declare_nat_symbols();
    symbols.declare_arith_symbols();
    symbols.declare_combinator_symbols();
    symbols.declare_fib_symbols();

    // // Fib
    // let fib_sym = symbols.declare1("fib", Polarity::Neg, Polarity::Pos);
    // let fib0_sym = symbols.declare1("fib₀", Polarity::Neg, Polarity::Pos);

    println!("{}", symbols);

    // rules
    let mut rules = RuleSet::new(&symbols);

    // Arith rules
    rules.arith_rules();
    rules.define_combinator_rules();
    rules.fib_rules();

    println!("{}", rules);
    // println!("{:?}", rules);

    println!();

    // net
    let mut net = Net::new(&symbols);

    // // 1 + 0 = 1
    // net.equations(|b| {
    //     let one = b.one();
    //     let r_fvar = b.output_fvar();
    //     let adder = b.adder(r_fvar.into(), one.into());

    //     let zero = b.zero();
    //     // ATTN: cannot "share" cells!! TODO how to avoid this?
    //     b.add(zero, adder);
    // });

    // // 1 + 2 = 3
    // println!("--- 1 + 2 = 3 ---");
    // net.equations(|b| {
    //     let one = b.one();
    //     let r_fvar = b.output_fvar();
    //     let adder = b.adder(r_fvar.into(), one.into());

    //     // ATTN: cannot "share" cells!! TODO how to avoid this?
    //     let two = b.two();
    //     b.add(two, adder);
    // });
    // println!("--- ---");

    // // 2 - 1 = 1
    // net.equations(|b| {
    //     let result = b.output_fvar();
    //     let one = b.one();
    //     let subtractor = b.subtractor(result.into(), one.into());

    //     let two = b.two();
    //     b.subtract(two, subtractor);
    // });

    // // 3 - 2 = 1
    // net.equations(|b| {
    //     let result = b.output_fvar();
    //     let two = b.two();
    //     let three = b.succ(two.into());
    //     let subtractor = b.subtractor(result.into(), three.into());

    //     let two = b.two();
    //     b.subtract(two, subtractor);
    // });

    // // 3 - 0 = 3
    // net.equations(|b| {
    //     let result = b.output_fvar();
    //     let two = b.two();
    //     let three = b.succ(two.into());
    //     let subtractor = b.subtractor(result.into(), three.into());

    //     let zero = b.zero();
    //     b.subtract(zero, subtractor);
    // });

    // // 1 - 2 = 0
    // net.equations(|b| {
    //     let result = b.output_fvar();
    //     let one = b.two();
    //     let subtractor = b.subtractor(result.into(), one.into());

    //     let two = b.two();
    //     b.subtract(two, subtractor);
    // });

    // // Duplicate Z
    // net.equations(|b| {
    //     let dup1 = b.output_fvar();
    //     let dup2 = b.output_fvar();
    //     let zero = b.zero();
    //     b.duplicate(zero.into(), dup1.into(), dup2.into());
    // });

    // // Duplicate One
    // net.equations(|b| {
    //     let dup1 = b.output_fvar();
    //     let dup2 = b.output_fvar();
    //     let one = b.one();
    //     b.duplicate(one.into(), dup1.into(), dup2.into());
    // });

    // // Duplicate Two
    // net.equations(|b| {
    //     let dup1 = b.output_fvar();
    //     let dup2 = b.output_fvar();
    //     let two = b.two();
    //     b.duplicate(two.into(), dup1.into(), dup2.into());
    // });

    // net.fib(0);
    // net.fib(1);
    // net.fib(2);
    // net.fib(4);
    // net.fib(8);
    // net.fib(16);
    net.fib(28);
    // net.fib(32);

    println!("{}", net);

    let mut runtime = Runtime::new(&rules, false);

    println!();

    // let net = runtime.run(net);
    let net = runtime.eval(net);

    println!("{}", net);
    println!();
}

fn a() {
    let mut value_a = None;
    let mut value_b = None;
    let mut value_c = None;
    rayon::scope(|s| {
        s.spawn(|s1| {
            // ^ this is the same scope as `s`; this handle `s1`
            //   is intended for use by the spawned task,
            //   since scope handles cannot cross thread boundaries.

            value_a = Some(22);
            b(s1);
        });

        s.spawn(|_| {
            value_c = Some(66);
        });
    });
    assert_eq!(value_a, Some(22));
    assert_eq!(value_b, Some(44));
    assert_eq!(value_c, Some(66));
}

fn b(s1: &Scope) {
    // the scope `s` will not end until all these tasks are done
    s1.spawn(|_| {});
}
