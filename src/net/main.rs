

use crate::net::dsl::{RuleDsl, EquationDsl, TermDsl};
use crate::net::rule::RuleBook;
use crate::net::runtime::Runtime;
use crate::net::term::{Net, BVar, FVar};
use crate::net::{symbol::SymbolBook, dsl::SymbolDsl};

use Polarity::*;

fn main() {
    // symbols
    let v = vec![1,2,3];
    let i1 = v.get_mut(1).unwrap();
    let i2 = &mut v[2];
    let i22 = &mut v[2..2];


    let mut symbols = SymbolBook::default();
    let Z = symbols.ctr0("Z");
    let S = symbols.ctr1("S", Neg);
    let ADD = symbols.fun2("add", Pos, Neg);

    // rules
    let mut rules = RuleBook::default();
    rules.rule(Z, ADD, |body|{
        let add0 = body.fun_port_0();
        let add1 = body.fun_port_1();
        body.connect(add0, add1);
    });
    rules.rule(S, ADD, |body|{
        let x = body.bvar('x' );
        let add__port_0 = body.fun_port_0();
        let S_x = body.cell1(S, x.into());
        body.bind(add__port_0, S_x);

        let z = body.cell0(Z);
        let add = body.cell2(ADD, x.into(), z.into());

        let s_port_0 = body.ctr_port_0();
        body.bind(s_port_0, add);
    });

    // net
    let mut net : Net<FVar,BVar> = Net::default();
    let z = net.cell0(Z);
    let r = net.fvar(FVar::new(Neg));
    let add = net.cell2(ADD, r.into(), z.into());
    let s = net.cell1(S, z.into());
    net.redex(s, add);

    // println!("NET = {:?}", net);

    let mut runtime = Runtime::new(symbols, rules);
    runtime.eval(net);
    // block_on(greeter());
    // println!("{:?}", eqn1);
    // println!("{:?}", eqn2);
    println!("Done!");
}
