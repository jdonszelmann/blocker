#![no_std]
//!
//! [![Codecov](https://img.shields.io/codecov/c/github/jonay2000/blocker?logo=codecov&style=for-the-badge)](https://codecov.io/gh/jonay200/blocker)
//! [![Docs.rs](https://img.shields.io/badge/docs.rs-blocker-66c2a5?style=for-the-badge&labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K)](https://docs.rs/blocker)
//! [![Crates.io](https://img.shields.io/crates/v/blocker?logo=rust&style=for-the-badge)](https://crates.io/crates/blocker)
//!
//! # Blocker!
//!
//! Blocker blocks. That's what it does, nothing more. Give it an async function and it waits until it's done. Forever.
//! Works in `#![no_std]` environments as long as alloc is available. Blocker does not rely on unsafe code.
//!
//! Enable the `thread_yield` feature to yield the current thread whenever an async function returns `Poll::pending`.
//!
//! # License
//!
//! This code is licensed under the [Apache 2.0 license](./LICENSE)

use core::future::Future;
use core::task::{Context, Poll};
use futures::task::noop_waker;

extern crate alloc;
use alloc::boxed::Box;

#[cfg(thread_yield)]
extern crate std;

pub trait Blocker {
    type Output;
    fn block(self) -> Self::Output;
}

/// Blocker is a trait implemented for any type which implements Future. When imported, calling
/// [`block`] on any future will halt the program until the future completes.
impl<T> Blocker for T
where
    T: Future,
{
    type Output = T::Output;

    fn block(self) -> Self::Output {
        block(self)
    }
}

/// block is the heart of the blocker crate. When called with any future as parameter it blocks the
/// program until the future completes. When futures return [`Pending`](core::task::Poll), the future
/// will just be repolled. When the `thread_yield` feature is enabled, a pending future will yield the
/// current thread. Note that this only works when std is available.
pub fn block<'a, T>(future: impl Future<Output = T>) -> T
where
    T: 'a,
{
    let waker = noop_waker();
    let mut ctx = Context::from_waker(&waker);

    let mut pinned = Box::pin(future);

    loop {
        if let Poll::Ready(i) = pinned.as_mut().poll(&mut ctx) {
            return i;
        } else {
            #[cfg(thread_yield)]
            std::thread::yield_now()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use alloc::rc::Rc;
    use core::ops::Deref;

    extern crate std;
    use std::sync::Mutex;

    async fn num() -> i64 {
        return 10;
    }

    #[test]
    pub fn test_block() {
        let f1 = num();
        let f2 = num();
        let f3 = num();

        assert_eq!(10, block(f2));
        assert_eq!(10, block(f1));
        assert_eq!(10, block(f3));
    }

    #[test]
    pub fn test_block_trait() {
        let f1 = num();
        let f2 = num();
        let f3 = num();

        assert_eq!(10, f2.block());
        assert_eq!(10, f1.block());
        assert_eq!(10, f3.block());
    }

    async fn rc(r: Rc<i64>) -> Rc<i64> {
        return r.clone();
    }

    #[test]
    pub fn test_rc() {
        let r = Rc::new(10);

        let f1 = rc(r.clone());

        let rcclone = r.deref();

        assert_eq!(&10, block(f1).deref());
        assert_eq!(&10, rcclone);
    }

    #[cfg_attr(miri, ignore)]
    async fn rc_mutex(r: Rc<Mutex<i64>>) -> Rc<Mutex<i64>> {
        let mut guard = r.deref().lock().unwrap();
        *guard = 15;
        drop(guard);

        return r;
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    pub fn test_rc_mutex() {
        let r = Rc::new(Mutex::new(10));

        let f1 = rc_mutex(r.clone());

        let original = r.deref();
        let blocked = block(f1);

        {
            let guard = blocked.deref().lock().unwrap();
            assert_eq!(15, *guard);
        }

        {
            let guard = original.deref().lock().unwrap();
            assert_eq!(15, *guard);
        }
    }
}
