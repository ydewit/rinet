mod symbol;
mod term;
mod net;
mod rule;

fn main() {
    println!("Hello, world!");
}

// newtype F a = F a			// actual variable
// struct F<A>(A);

// data U = Used
// enum U {
//     Used,
// }

// trait LSymantics {
//     type Repr<Hi, Ho, T>;

//     // int :: Int -> repr hi hi Int
//     fn int<Hi>(i: i32) -> Self::Repr<Hi, Hi, i32>;

//     // add :: repr hi h Int -> repr h ho Int -> repr hi ho Int
//     fn add<Hi, H, Ho>(
//         x: Self::Repr<Hi, H, i32>,
//         y: Self::Repr<H, Ho, i32>,
//     ) -> Self::Repr<Hi, Ho, i32>;

//     // z   :: repr (F a,h) (U,h) a
//     fn z<A, H>() -> Self::Repr<(F<A>, H), (U, H), A>;

//     // s   :: repr hi ho a -> repr (any,hi) (any,ho) a
//     fn s<A, Hi, Ho, Any>(x: Self::Repr<Hi, Ho, A>) -> Self::Repr<(Any, Hi), (Any, Ho), A>;

//     // app :: repr hi h (a->b) -> repr h ho a -> repr hi ho b
//     fn app<Hi, H, Ho, A, B, L>(
//         x: Self::Repr<Hi, H, L>,
//         y: Self::Repr<H, Ho, A>,
//     ) -> Self::Repr<Hi, Ho, B>
//     where
//         L: Fn(A) -> B;
// }

// // The reason we separate out 'lam' is to expose the type variables
// // hi and ho in the class head. A particular instance might be able to attach
// // constraints to hi and ho. The instance for the R interpreter
// // indeed attaches the HiHo constraint.
// // class LinearL repr hi ho where
// trait LinearL: LSymantics {

//     // lam :: repr (F a,hi) (U,ho) b  -> repr hi ho (a->b)
//     fn lam<Hi, Ho, A, B, L>(repr: Self::Repr<(F<A>, Hi), (U, Ho), B>) -> Self::Repr<Hi, Ho, L>
//     where
//         L: Fn(A) -> B;
// }

// //tl1 :: (LSymantics repr) => repr hi hi Int
// //tl1 = add (int 1) (int 2)
// fn tl1<Hi, L: LSymantics>() -> L::Repr<Hi, Hi, i32> {
//     L::add(L::int(1), L::int(2))
// }

// // tl2o :: (LSymantics repr) => repr (F Int, h) (U, h) (Int -> Int)
// // tl2o = lam (add z (s z))
// fn tl2o<Hi, L: LSymantics>() -> L::Repr<Hi, Hi, i32> {
//     let a =L::add(L::z(), L::z());
//     L::lam()
// }



// fn sample_net<N: Net>() -> N:: {
//     Net::bvar(|wire| {
//         Net::redex(
//             Cell::cell2(Symbol::symbol(), Port::bvar(), Port::bvar()),
//             Cell::cell2(Symbol::symbol(), Port::bvar(), Port::bvar()),
//         );
//     })
// }
