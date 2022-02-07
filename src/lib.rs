#![no_std]
//! ## Polonius_workaround
//!
//!

/// Extension trait to implement described functionality
pub trait PoloniusExt {
    fn try_get_with<'a, F, R>(&'a mut self, f: F) -> Result<&'a R, &'a mut Self>
    where
        F: for<'x> FnOnce(&'x mut Self) -> Option<&'x R>,
    {
        let ptr = self as *mut _;
        let result = f.call(self);
        result.ok_or_else(|| unsafe { &mut *ptr })
    }

    fn try_get_with_mut<'a, F, R>(&'a mut self, f: F) -> Result<&'a mut R, &'a mut Self>
    where
        F: for<'x> FnOnce(&'x mut Self) -> Option<&'x mut R>,
    {
        let ptr = self as *mut _;
        let result = f.call(self);
        result.ok_or_else(|| unsafe { &mut *ptr })
    }

    fn try_get_with2<'a, F>(
        &'a mut self,
        f: F,
    ) -> Result<<F as MyFnOnce<&'a mut Self>>::Output, &'a mut Self>
    where
        for<'x> F: MyFnOnce<&'x mut Self>,
    {
        let ptr = self as *mut _;
        let result = f.call(self);
        result.ok_or_else(|| unsafe { &mut *ptr })
    }
}
impl<T> PoloniusExt for T {}

pub trait MyFnOnce<T> {
    type Output;

    fn call(self, arg: T) -> Option<Self::Output>;
}

impl<T, F, O> MyFnOnce<T> for F
where
    F: FnOnce(T) -> Option<O>,
{
    type Output = O;

    fn call(self, arg: T) -> Option<Self::Output> {
        self(arg)
    }
}

// //todo
// struct Node(u64, [Option<Box<Node>>; 1]);
// fn main() {
//     let mut root = Node(0, [None]);
//     let mut current = &mut root;
//     for _ in 0..1 {
//         // works
//         // if let Some(ref mut id) = current.1[0] {
//         //     current = id;
//         // }
//         // doesn't work
//         // if let Some(ref mut id) = current.1.get_mut(0).unwrap() {
//         //     current = id;
//         // }
//         let cl = tmp2(|x: &'_ mut Node| -> Option<&'_ mut Node> {
//             x.1.get_mut(0).unwrap().as_deref_mut()
//         });
//         // let cl = hrtb!( for<'x> |x:&'x mut Node| -> Option<&'x mut Node> {
//         //     x.1.get_mut(0).unwrap().as_deref_mut()
//         // });
//         current = current.try_get_with_mut(cl).unwrap_or_else(|x| x);
//     }
// }
//
// fn tmp2<X>(f: X) -> X
// where
//     X: for<'x> FnOnce(&'x mut Node) -> Option<&'x mut Node>,
// {
//     f
// }
//
// struct VM {/* ip, stack, etc. */}
// struct Test<'a>(&'a str);
// impl VM {
//     fn step(&mut self) -> Option<&str> {
//         // decode and execute a single potentially fallible bytecode op
//         // oh, an error occured:
//         return Some("(•_• ?)");
//     }
//
//     pub fn run(mut self: &mut Self) -> Result<(), &str> {
//         // stepping through the program
//         loop {
//             // stop running if something goes wrong
//             // self.step?;
//             self = match self.try_get_with(Self::step) {
//                 Ok(x) => return Err(x),
//                 Err(x) => x,
//             };
//         }
//     }
//
//     fn step2(&mut self) -> Option<Test<'_>> {
//         // decode and execute a single potentially fallible bytecode op
//         // oh, an error occured:
//         return Some(Test("(•_• ?)"));
//     }
//
//     pub fn run2(mut self: &mut Self) -> Result<(), Test<'_>> {
//         // stepping through the program
//         loop {
//             // stop running if something goes wrong
//             // self.step?;
//             self = match self.try_get_with(Self::step2) {
//                 Ok(x) => return Err(x),
//                 Err(x) => x,
//             };
//         }
//     }
// }
//
// use std::collections::LinkedList;
// trait LinkedListExt<T> {
//     fn find_or_create<F, C>(&mut self, predicate: F, create: C) -> &mut T
//     where
//         F: FnMut(&T) -> bool,
//         C: FnMut() -> T;
// }
//
// impl<T: 'static> LinkedListExt<T> for LinkedList<T> {
//     fn find_or_create<F, C>(&mut self, mut predicate: F, mut create: C) -> &mut T
//     where
//         F: FnMut(&T) -> bool,
//         C: FnMut() -> T,
//     {
//         // let cl = ;
//         self.try_get_with(tmp(|x: &'_ mut Self| x.iter_mut().find(|e| predicate(&*e))))
//             .unwrap_or_else(|x| {
//                 x.push_back(create());
//                 x.back_mut().unwrap()
//             })
//     }
// }
//
// fn tmp<X, T>(f: X) -> X
// where
//     X: for<'x> FnOnce(&'x mut LinkedList<T>) -> Option<&'x mut T>,
// {
//     f
// }
