#![no_std]
//! This crate provides api to solve borrow checker errors caused by limitation of current rust borrow checked.
//! There exists next version of rust borrow checker called Polonius that is supposed to remove such limitations,
//! but it is permanently unstable with no current plans for stabilization.
//! And quite often the only way to work around those limitations on stable rust without some
//! kind of regression is with unsafe code. But it is annoying to be required to use unsafe for
//! such simple things, and it is too easy to get wrong,
//! not to mention it is just delayed bomb waiting for some uncareful refactoring to release it.
//!
//! All functionality is provided via `PoloniusExt` extension trait.
//! It has 3 methods:
//! - `try_get_with`/`try_get_mut_with` work for simple cases where you need to return
//!     shared/mutable reference respectively. It should just work and in most cases that is enough.
//!     But sometimes you need to return not a reference but some another type that contains a reference.
//!     Thats when you need
//! - `try_get_with2` It allows you to return any type with reference inside,
//!     but due to rust type inference bugs around HRTBs it is a bit annoying to use.
//!     See its docs for more details.
//!
//! As far as author knows it allows to solve any king of borrow checker issues that would be solved by Polonius.
//! Although you still need to be confident enough in Rust to know that your code
//! is actually correct and just not accepted by current borrow checker.
//! So although with this crate you do not actually need Polonius anymore,
//! it is still a nice thing to have in general.
//!
//! The code compiles since Rust 1.0 but
//! but `try_get_with2` actually works only since Rust 1.41.
//!
//! And here is a real example that you couldn't work around without significant performance regression or unsafe.
//! ```rust,compile_fail
//! # use std::collections::LinkedList;
//! trait LinkedListExt<T> {
//!     fn find_or_create<F, C>(&mut self, predicate: F, create: C) -> &mut T
//!     where
//!         F: FnMut(&T) -> bool,
//!         C: FnMut() -> T;
//! }
//!
//! impl<T: 'static> LinkedListExt<T> for LinkedList<T> {
//!     fn find_or_create<F, C>(&mut self, mut predicate: F, mut create: C) -> &mut T
//!     where
//!         F: FnMut(&T) -> bool,
//!         C: FnMut() -> T,
//!     {
//!         if let Some(x) = self.iter_mut().find(|e| predicate(&*e)){
//!             return x;
//!         }
//!         self.push_back(create());
//!         self.back_mut().unwrap()
//!     }
//! }
//! ```
//! Now with this crate you just wrap two branches into a closures then call `try_get_mut_with` and voila - it works
//! ```rust
//! # use std::collections::LinkedList;
//! # use polonius_workaround::PoloniusExt;
//! # trait LinkedListExt<T> {
//! #     fn find_or_create<F, C>(&mut self, predicate: F, create: C) -> &mut T
//! #     where
//! #         F: FnMut(&T) -> bool,
//! #         C: FnMut() -> T;
//! # }
//! impl<T: 'static> LinkedListExt<T> for LinkedList<T> {
//!     fn find_or_create<F, C>(&mut self, mut predicate: F, mut create: C) -> &mut T
//!     where
//!         F: FnMut(&T) -> bool,
//!         C: FnMut() -> T,
//!     {
//!         self.try_get_mut_with(|x| x.iter_mut().find(|e| predicate(&*e)))
//!             .unwrap_or_else(|x| {
//!                 x.push_back(create());
//!                 x.back_mut().unwrap()
//!             })
//!     }
//! }
//! ```

/// Extension trait to implement described functionality.
///
/// The general idea is to store mutable reference value in raw pointer while the closure executes.
/// If the closure returned `Some` then we just forget that pointer and everything goes as usual,
/// the mere existence of raw pointer can't break anything.
/// If the closure returned `None` we know that any borrow derived from `& mut Self` can't be alive
/// at that point so we can soundly restore the original `&mut Self`.
pub trait PoloniusExt {
    ///```rust
    /// use polonius_workaround::PoloniusExt;
    /// struct VM {/* ip, stack, etc. */}
    /// impl VM {
    ///     fn step(&mut self) -> Result<(),&str> {
    ///         // decode and execute a step
    ///         // oh, an error occured:
    ///         return Err("(•_• ?)");
    ///     }
    ///
    ///     pub fn run(mut self: &mut Self) -> Result<(), &str> {
    ///         loop {
    ///             // this doesn't work
    ///             // x.step()?;
    ///             self = match self.try_get_with(|x| x.step().err()) {
    ///                 Ok(x) => return Err(x),
    ///                 Err(x) => x,
    ///             };
    ///             
    ///         }
    ///     }
    /// }
    /// ```
    fn try_get_with<'a, F, R: ?Sized>(&'a mut self, f: F) -> Result<&'a R, &'a mut Self>
    where
        F: for<'x> FnOnce(&'x mut Self) -> Option<&'x R>,
    {
        let ptr = self as *mut _;
        let result = f.call(self);
        result.ok_or_else(|| unsafe { &mut *ptr })
    }

    /// This crate works not only when you return references, but also when you need to
    /// reassign reference to the same variable in the loop  
    /// ```rust
    /// use polonius_workaround::PoloniusExt;
    /// struct Node(u64, [Option<Box<Node>>; 1]);
    /// impl Node{
    ///     fn get_mut(&mut self,idx:usize) -> Option<&mut Node>{
    ///         self.1.get_mut(0).unwrap().as_deref_mut()
    ///     }
    /// }
    /// fn main() {
    ///     let mut root = Node(0, [None]);
    ///     let mut current = &mut root;
    ///     for _ in 0..2 {
    ///         // doesn't work
    ///         // if let Some(ref mut id) = current.get_mut(0) {
    ///         //     current = id;
    ///         // }
    ///         current = current.try_get_mut_with(|x| x.get_mut(0) )
    ///             .unwrap_or_else(|x| x);
    ///     }
    /// }
    fn try_get_mut_with<'a, F, R: ?Sized>(&'a mut self, f: F) -> Result<&'a mut R, &'a mut Self>
    where
        F: for<'x> FnOnce(&'x mut Self) -> Option<&'x mut R>,
    {
        let ptr = self as *mut _;
        let result = f.call(self);
        result.ok_or_else(|| unsafe { &mut *ptr })
    }

    /// This function relies on quite advanced trait magic and rust type inference is still bugged
    /// around such cases so it just works only if you pass it a standalone function as a parameter.
    /// It still can work in more complex cases but it requires some hacks.
    ///
    /// Here is simple example that just works:
    /// ```rust
    /// use polonius_workaround::PoloniusExt;
    /// struct VM {/* ip, stack, etc. */}
    /// struct Test<'a>(&'a str);
    /// impl VM {
    ///     fn step(&mut self) -> Option<Test<'_>> {
    ///         return Some(Test("(•_• ?)"));
    ///     }
    ///
    ///     pub fn run(mut self: &mut Self) -> Option<Test<'_>> {
    ///         loop {
    ///             // that doesn't work
    ///             // self.step()?;
    ///             self = match self.try_get_with2(Self::step) {
    ///                 Ok(x) => return Some(x),
    ///                 Err(x) => x,
    ///             };
    ///         }
    ///     }
    /// }
    /// ```
    /// but imagine if `step` needs a parameter, then we have to use a closure.
    /// And now we need to do some dancing.
    /// ```rust
    /// # use polonius_workaround::PoloniusExt;
    /// # struct VM {/* ip, stack, etc. */}
    /// # struct Test<'a>(&'a str);
    /// # impl VM {
    ///     fn step(&mut self, arg: usize) -> Option<Test<'_>> {
    ///         if arg == 0 { return None }
    ///         return Some(Test("(•_• ?)"));
    ///     }
    ///
    ///     pub fn run(mut self: &mut Self) -> Option<Test<'_>> {
    ///         loop {
    ///             // this should have worked, but rust type inference is not powerful enough yet
    ///             // self = match self.try_get_with2(|x| x.step(0) ) {
    ///             // so we need a helper function to help type inference
    ///             fn helper<X>(f: X) -> X
    ///             where
    ///                 X: for<'x> FnOnce(&'x mut VM) -> Option<Test<'x>>,
    ///             {
    ///                 f
    ///             }
    ///             // now we wrap closure in it and compiler finally can do it's job
    ///             self = match self.try_get_with2(helper(|x| x.step(0))) {
    ///                 Ok(x) => return Some(x),
    ///                 Err(x) => x,
    ///             };
    ///         }
    ///     }
    /// # }
    /// ```
    /// Quite ugly as you can see, but blame the compiler here.
    /// Nevertheless you can significantly reduce that ugliness with a helper crate `higher-order-closure`.
    /// With its `hrtb!` macro that line will become just:
    /// ```rust,compile_fail
    /// # struct Test;
    /// # impl Test{
    /// #     fn test(&mut self) {
    ///             let result = self.try_get_with2(hrtb!(
    ///                 for<'x> |x:&'x mut VM| -> Option<Test<'x>> { x.step(0) }
    ///             ));
    ///             self = match result {
    /// #  }}}
    /// ```
    ///
    fn try_get_with2<'a, F>(
        &'a mut self,
        f: F,
    ) -> Result<<F as FnOnceReturningOption<&'a mut Self>>::Output, &'a mut Self>
    where
        for<'x> F: FnOnceReturningOption<&'x mut Self>,
    {
        let ptr = self as *mut _;
        let result = f.call(self);
        result.ok_or_else(|| unsafe { &mut *ptr })
    }
}
impl<T: ?Sized> PoloniusExt for T {}

/// Helper trait for `PoloniusExt::try_get_with2`
pub trait FnOnceReturningOption<T> {
    type Output;

    fn call(self, arg: T) -> Option<Self::Output>;
}

impl<T, F, O> FnOnceReturningOption<T> for F
where
    F: FnOnce(T) -> Option<O>,
{
    type Output = O;

    fn call(self, arg: T) -> Option<Self::Output> {
        self(arg)
    }
}
