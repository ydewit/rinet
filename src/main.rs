// #![feature(once_cell)]
#![feature(generic_const_exprs)]

// mod net;
mod inet;

fn main() {
    inet_main()
}

use crate::inet::{net::{Net, NetItem}, rule::{RuleBook, RulesItem}, runtime::Runtime, symbol::SymbolBook, Polarity};

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
        let r_fvar = net.fvar();
        let add_ptr = net.cell2(add_sym, r_fvar.into(), s.into());
        net.redex(s.into(), add_ptr.into());
    });

    println!("{}", NetItem::new(&symbols, &net));
    println!();
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

    println!("{}", RulesItem{ symbols: &symbols, book: &rules });
    println!();
    println!("{:?}", rules);

    println!();
    let mut runtime = Runtime::new(&symbols, &rules);
    runtime.eval(net);
}
// add_cell: 10000000000010_0100000000000000000000000_0000000000000000000000001

// fn test(rules: &RuleBook, net: &Net) {

//     let a : Vec<Equation> = Vec::new();

//     let b = a.par_iter().flat_map(|&eqn|{
//         eval_equation(rules, net, eqn)
//     }).collect();
// }

// async fn eval_equation(rules: &RuleBook, net: &Net, eqn: Equation) -> Option<Equation> {
//     match eqn.get_kind() {
//         EquationKind::Redex => {
//             let ctr = net.get_cell(eqn.get_redex_ctr());
//             let fun = net.get_cell(eqn.get_redex_fun());

//             rules.rewrite(net, eqn_ptr, ctr, fun)
//         },
//         EquationKind::Bind => {
//             let var_ptr = eqn.get_bind_var();
//             let cell_ptr = eqn.get_bind_cell();
//             match cell_ptr.get_polarity() {
//                 Polarity::Pos => {
//                     match net.try_set_var(var_ptr, cell_ptr) {
//                         Some(fun_cell_ptr) => {
//                             eval(rules, net, Equation::redex(cell_ptr, fun_cell_ptr))
//                         },
//                         None => None // setting was done,
//                     }
//                 },
//                 Polarity::Neg => {
//                     match net.try_get_var(var_ptr) {
//                         Some(ctr_cell_ptr) => {
//                             eval(rules, net, Equation::redex(ctr_cell_ptr, cell_ptr))
//                         },
//                         None => Some(eqn), // Keep waiting
//                     }
//                 },
//             }
//         },
//         EquationKind::Connect => {
//             let left_ptr = net.try_get_var(eqn.get_connect_left());
//             let right_ptr = net.try_get_var(eqn.get_connect_right());
//             match (left_ptr, right_ptr){
//                 (None, None) => Some(eqn),
//                 (None, Some(cell_ptr)) => eval(rules, net, Equation::bind(eqn.get_connect_left(), cell_ptr)),
//                 (Some(cell_ptr), None) => eval(rules, net, Equation::bind(eqn.get_connect_right(), cell_ptr)),
//                 (Some(left_cell_ptr), Some(right_cell_ptr)) => {
//                     match (left_cell_ptr.get_polarity(), right_cell_ptr.get_polarity()) {
//                         (Polarity::Pos, Polarity::Neg) => eval(rules, net, Equation::redex(left_cell_ptr, right_cell_ptr)),
//                         (Polarity::Neg, Polarity::Pos) => eval(rules, net, Equation::redex(right_cell_ptr, left_cell_ptr)),
//                         _ => panic!()
//                     }
//                 },
//             }
//         },
//     }
// }
