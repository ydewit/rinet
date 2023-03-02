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
    let z_sym = symbols.add_symbol0("Z", Polarity::Pos);
    let s_sym = symbols.add_symbol1("S", Polarity::Pos, Polarity::Neg);
    let add_sym = symbols.add_symbol2("add", Polarity::Neg, Polarity::Pos, Polarity::Neg);
    println!("{}", symbols);

    // net
    let net = Net::new(|net| {
        let z = net.cell0(z_sym);
        let s = net.cell1(s_sym, z.into());
        let r_fvar = net.var();
        let add_ptr = net.cell2(add_sym, r_fvar.into(), s.into());
        net.redex(s.into(), add_ptr.into());
    });

    println!("{:?}", net);

    // rules

    // Z >< add
    let mut rules = RuleBook::new();
    rules.new_rule(z_sym, add_sym, |builder| {
        let add0 = builder.fun_port_0();
        let add1 = builder.fun_port_1();
        builder.connect(add0.into(), add1.into());
    });
    // S >< add
    rules.new_rule(s_sym, add_sym, |builder| {
        let x = builder.var();
        let fun_0 = builder.fun_port_0();
        let S_x = builder.cell1(s_sym, x.into());
        builder.bind(fun_0.into(), S_x);

        let fun_1 = builder.fun_port_1();
        let add = builder.cell2(add_sym, x.into(), fun_1.into());

        let s_port_0 = builder.ctr_port_0();
        builder.bind(s_port_0.into(), add);
    });

    println!("{}", rules.display_rules(&symbols));
    println!();
    println!("{:?}", rules);

    println!();

    let mut runtime = Runtime::new(&symbols, &rules);

    println!("{}", net.display_net(&symbols));
    println!();

    let net = runtime.eval(net);

    println!("{}", net.display_net(&symbols));
    println!();
}
