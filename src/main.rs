// #![feature(once_cell)]
#![feature(generic_const_exprs)]

// mod net;
mod inet;

fn main() {
    inet_main()
}

use crate::inet::{net::Net, rule::RuleBook, runtime::Runtime, symbol::SymbolBook, Polarity};

pub fn inet_main() {
    // symbols
    let mut symbols = SymbolBook::new();

    // Nat
    let z_sym = symbols.declare0("Z", Polarity::Pos);
    let s_sym = symbols.declare1("S", Polarity::Pos, Polarity::Neg);

    // Add
    let add_sym = symbols.declare2("add", Polarity::Neg, Polarity::Pos, Polarity::Neg);

    // Sub
    let sub_sym = symbols.declare2("sub", Polarity::Neg, Polarity::Pos, Polarity::Neg);
    let sub0_sym = symbols.declare2("sub₀", Polarity::Neg, Polarity::Pos, Polarity::Neg);

    // Dup
    let dup_sym = symbols.declare2("dup", Polarity::Neg, Polarity::Pos, Polarity::Pos);

    // Fib
    let fib_sym = symbols.declare1("fib", Polarity::Neg, Polarity::Pos);
    let fib0_sym = symbols.declare1("fib₀", Polarity::Neg, Polarity::Pos);

    println!("{}", symbols);

    // rules
    let mut rules = RuleBook::new(&symbols);

    // # Add

    // add(x₁ x₂)=Z  ⟶  x₁ = x₂
    rules.rule(z_sym, add_sym, |builder| {
        let add0 = builder.fun_port_0();
        let add1 = builder.fun_port_1();
        builder.connect(add0.into(), add1.into());
    });
    // add(x₁ x₂)=(S y)  ⟶  x1=(S w), y=(add w x₂)
    rules.rule(s_sym, add_sym, |builder| {
        let x = builder.var();
        let fun_0 = builder.fun_port_0();
        let S_x = builder.cell1(s_sym, x.into());
        builder.bind(fun_0.into(), S_x);

        let fun_1 = builder.fun_port_1();
        let add = builder.cell2(add_sym, x.into(), fun_1.into());

        let s_port_0 = builder.ctr_port_0();
        builder.bind(s_port_0.into(), add);
    });

    // # Sub

    // (Sub l0 l1) = Z       ⟶  l0 = l1
    rules.rule(z_sym, sub_sym, |builder| {
        let l0 = builder.fun_port_0();
        let l1 = builder.fun_port_1();
        builder.connect(l0.into(), l1.into());
    });
    // (Sub l0 l1) = (S r0)  ⟶  (Sub₀ l0 r0) = l1
    rules.rule(s_sym, sub_sym, |builder| {
        let l0 = builder.fun_port_0();
        let l1 = builder.fun_port_1();
        let r0 = builder.ctr_port_0();

        let sub0 = builder.cell2(sub0_sym, l0.into(), r0.into());
        builder.bind(l1.into(), sub0.into());
    });

    // (Sub₀ l0 l1) = Z       ⟶  l0 = (S l1)
    rules.rule(z_sym, sub0_sym, |builder| {
        let l0 = builder.fun_port_0();
        let l1 = builder.fun_port_1();

        let s = builder.cell1(s_sym, l1.into());
        builder.bind(l0.into(), s.into());
    });
    // (Sub₀ l0 l1) = (S r0)  ⟶  (Sub l0 r0) = l1
    rules.rule(s_sym, sub0_sym, |builder| {
        let l0 = builder.fun_port_0();
        let l1 = builder.fun_port_1();
        let r0 = builder.ctr_port_0();

        let sub = builder.cell2(sub_sym, l0.into(), r0.into());
        builder.bind(l1.into(), sub.into());
    });

    // # Dup
    rules.rule(z_sym, dup_sym, |b| {
        let r0 = b.fun_port_0();
        let z0 = b.cell0(z_sym);
        b.bind(r0.into(), z0.into());

        let r1 = b.fun_port_1();
        let z1 = b.cell0(z_sym);
        b.bind(r1.into(), z1.into());
    });
    rules.rule(s_sym, dup_sym, |b| {
        let x0 = b.var();
        let x1 = b.var();

        let l0 = b.ctr_port_0();
        let dup = b.cell2(dup_sym, x0.into(), x1.into());
        b.bind(l0.into(), dup.into());

        let r0 = b.fun_port_0();
        let s0 = b.cell1(s_sym, x0.into());
        b.bind(r0.into(), s0.into());

        let r1 = b.fun_port_1();
        let s1 = b.cell1(s_sym, x1.into());
        b.bind(r1.into(), s1.into());
    });

    println!("{}", rules);
    // println!("{:?}", rules);

    println!();

    // net
    let mut net = Net::new(&symbols);

    // 1 + 0 = 1
    net.equations(|b|{
        let z = b.cell0("Z");
        let s = b.cell1("S", z.into());
        let r_fvar = b.fvar();
        let add_ptr = b.cell2("add", r_fvar.into(), s.into());

        // ATTN: cannot "share" cells!! TODO how to avoid this?
        let z2 = b.cell0("Z");
        b.redex(z2.into(), add_ptr.into());
    });

    // 1 + 1 = 2
    net.equations(|b|{
        let z = b.cell0("Z");
        let s = b.cell1("S", z.into());
        let r_fvar = b.fvar();
        let add_ptr = b.cell2("add", r_fvar.into(), s.into());

        // ATTN: cannot "share" cells!! TODO how to avoid this?
        let z2 = b.cell0("Z");
        let s2 = b.cell1("S", z2.into());

        b.redex(s2.into(), add_ptr.into());
    });

    println!("{}", net);

    let mut runtime = Runtime::new(&symbols, &rules);

    println!();

    let net = runtime.eval(net);

    println!("{}", net);
    println!();
}
