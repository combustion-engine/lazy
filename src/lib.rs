//! Lazy initialization based on statically defined associated functions rather than callbacks.
//!
//! It also also errors in initialization, which other lazy libraries do not.

#![deny(missing_docs)]
#![allow(unknown_lints, inline_always)]

use std::ptr;
use std::marker::PhantomData;
use std::cell::UnsafeCell;

enum State<T> {
    Unevaluated,
    InProgress,
    Evaluated(T),
}

/// Lazy data structure
pub struct Lazy<T, E = ()> {
    inner: UnsafeCell<State<T>>,
    _error_marker: PhantomData<E>
}

impl<T, E> Default for Lazy<T, E> {
    #[inline(always)]
    fn default() -> Lazy<T, E> {
        Lazy::new()
    }
}

/// Defines the "constructor" for a `Lazy` instance.
pub trait LazyInit<T, E = ()> {
    /// Initialize the value
    fn init() -> Result<T, E>;
}

impl LazyInit<()> for () {
    #[inline(always)]
    fn init() -> Result<(), ()> {
        Ok(())
    }
}

impl<T, E> Lazy<T, E> {
    /// Create a new uninitialized lazy instance
    #[inline(always)]
    pub fn new() -> Lazy<T, E> {
        Lazy {
            inner: UnsafeCell::new(State::Unevaluated),
            _error_marker: PhantomData
        }
    }

    /// Set the internal value.
    ///
    /// # Panics
    ///
    /// Panics if the evaluation is already in progress.
    pub unsafe fn set(&self, value: T) {
        match *self.inner.get() {
            State::InProgress => { panic!("Lazy evaluation called from itself."); }
            _ => { *self.inner.get() = State::Evaluated(value) }
        }
    }

    /// Returns `Some(&T)` if the instance has been evaluated, `None` otherwise
    pub fn get_maybe(&self) -> Option<&T> {
        if let State::Evaluated(ref val) = *unsafe { &*self.inner.get() } {
            Some(val)
        } else {
            None
        }
    }

    /// Returns `Some(&mut T)` if the instance has been evaluated, `None` otherwise
    pub fn get_maybe_mut(&self) -> Option<&mut T> {
        if let State::Evaluated(ref mut val) = *unsafe { &mut *self.inner.get() } {
            Some(val)
        } else {
            None
        }
    }
}

impl<T, E> Lazy<T, E> where T: LazyInit<T, E> {
    #[inline(never)]
    fn evaluate(&self) -> Result<(), E> {
        unsafe {
            match *self.inner.get() {
                State::Evaluated(_) => return Ok(()),
                State::InProgress => panic!("Lazy evaluation called from itself."),
                _ => {}
            }

            match ptr::replace(self.inner.get(), State::InProgress) {
                State::Unevaluated => {
                    *self.inner.get() = State::Evaluated(<T as LazyInit<T, E>>::init()?);
                },
                _ => unreachable!()
            }
        }

        Ok(())
    }

    /// Evaluates the instance and returns a reference to the result.
    ///
    /// If the instance was already eveluated, the previous value is returned.
    pub fn get(&self) -> Result<&T, E> {
        self.evaluate()?;

        if let State::Evaluated(ref val) = *unsafe { &*self.inner.get() } {
            return Ok(val);
        }

        unreachable!()
    }

    /// Evaluates the instance and returns a mutable reference to the result.
    ///
    /// If the instance was already eveluated, the previous value is returned.
    pub fn get_mut(&mut self) -> Result<&mut T, E> {
        self.evaluate()?;

        if let State::Evaluated(ref mut val) = *unsafe { &mut *self.inner.get() } {
            return Ok(val);
        }

        unreachable!()
    }
}