## Polonius_workaround
This crate provides api to solve borrow checker errors caused by limitation of current rust borrow checked.
There exists next version of rust borrow checker called Polonius that is supposed to remove such limitation,
but it is permanently unstable with no current plans for stabilization. 
And quite often the only way to work around those limitations on stable rust without some 
kind of regression is with unsafe code. But it is too easy to get wrong, not to mention that it is just annoying that 
your perfectly correct code that is supposed to just work requires some black magic to make it work. 
And that unencapsulated piece of unsafe code is just bomb waiting for some uncarefull refactoring to release it. 
So this crate provides simple and logical safe api over such unsafe code. 