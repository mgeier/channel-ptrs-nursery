//! Pointers to channels.
//!
//! These are needed in some C APIs, do not use this in pure Rust code!

#![no_std]
#![forbid(clippy::undocumented_unsafe_blocks)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, vec::Vec};

use core::ops::Deref;

/// Pointers to audio channels.
///
/// # Safety
///
/// Implementations of this trait must ensure that the returned pointer points to
/// `self.channels()` valid pointers which in turn point to `self.frames()` valid elements
/// of type `Self::Item`.
/// The returned pointer must be non-null, even if there are zero channels.
/// If there are non-zero channels, all channel pointers must be non-null,
/// even if there are zero frames.
pub unsafe trait ChannelPtrs {
    type Item;

    fn frames(&self) -> usize;
    fn channels(&self) -> u16;
    fn as_ptr(&self) -> *const *const Self::Item;
    fn as_slice(&self) -> &[*const Self::Item] {
        // SAFETY: See docstring.
        unsafe { core::slice::from_raw_parts(self.as_ptr(), self.channels().into()) }
    }
}

// The provided impls for [_; _], &[_] and Vec<_> never allocate memory,
// ChannelPtrsBoxed uses a dynamic allocation.
pub trait IntoChannelPtrs {
    type Item;
    type IntoPtrs: ChannelPtrs<Item = Self::Item>;

    fn into_channel_ptrs(self) -> Self::IntoPtrs;
}

// Blanket implementation.
impl<P: ChannelPtrs> IntoChannelPtrs for P {
    type Item = <P as ChannelPtrs>::Item;
    type IntoPtrs = P;

    fn into_channel_ptrs(self) -> Self::IntoPtrs {
        self
    }
}

// Invariant: All pointers point to `frames` initialized elements of type `T`.
pub struct ChannelPtrsArray<T, const N: usize> {
    frames: usize,
    channels: [*const T; N],
}

// SAFETY: All pointers point to `self.frames` `T`s each.
unsafe impl<T, const N: usize> ChannelPtrs for ChannelPtrsArray<T, N> {
    type Item = T;

    fn frames(&self) -> usize {
        self.frames
    }

    fn channels(&self) -> u16 {
        N.try_into().unwrap()
    }

    fn as_ptr(&self) -> *const *const Self::Item {
        self.channels.as_ptr()
    }
}

// This can be chosen arbitrarily as trade-off between stack usage and convenience.
const MAX_CHANNELS_FROM_SLICE: usize = 16;

impl<T, Inner: Deref<Target = [T]>> IntoChannelPtrs for &[Inner] {
    type Item = T;

    type IntoPtrs = ChannelPtrsPartialArray<T, MAX_CHANNELS_FROM_SLICE>;

    fn into_channel_ptrs(self) -> Self::IntoPtrs {
        let frames = self
            .iter()
            .map(Deref::deref)
            .map(<[T]>::len)
            .reduce(|a, b| {
                assert_eq!(a, b, "all channels must have equal length");
                a
            })
            .unwrap_or(0);
        let channels: u16 = self.len().try_into().expect("slice too long");
        if MAX_CHANNELS_FROM_SLICE < channels.into() {
            panic!(
                "Too many channels for automatic conversion: {channels} \
                (maximum: {MAX_CHANNELS_FROM_SLICE})\nUse ChannelPtrsBoxed instead."
            );
        }
        let mut storage = [core::ptr::dangling(); MAX_CHANNELS_FROM_SLICE];
        // NB: zip() stops when one of the iterators is exhausted.
        for (src, dst) in self.iter().zip(storage.iter_mut()) {
            *dst = src.as_ptr();
        }
        Self::IntoPtrs {
            frames,
            channels,
            storage,
        }
    }
}

// Invariant: The first `channels` pointers point to `frames` initialized elements of type `T`.
pub struct ChannelPtrsPartialArray<T, const N: usize> {
    frames: usize,
    channels: u16,
    storage: [*const T; N],
}

// SAFETY: The first `self.channels` pointers point to `self.frames` `T`s each.
unsafe impl<T, const N: usize> ChannelPtrs for ChannelPtrsPartialArray<T, N> {
    type Item = T;

    fn frames(&self) -> usize {
        self.frames
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn as_ptr(&self) -> *const *const Self::Item {
        self.storage[..self.channels.into()].as_ptr()
    }
}

// NB: we cannot implement the more generic `Outer: Deref<Target = [Inner]>`
// because of conflicting implementations for `[Inner; N]`.
#[cfg(feature = "alloc")]
impl<T, Inner: Deref<Target = [T]>> IntoChannelPtrs for Vec<Inner> {
    type Item = T;

    type IntoPtrs = ChannelPtrsPartialArray<T, MAX_CHANNELS_FROM_SLICE>;

    fn into_channel_ptrs(self) -> Self::IntoPtrs {
        self.deref().into_channel_ptrs()
    }
}

impl<T, Inner: Deref<Target = [T]>, const N: usize> IntoChannelPtrs for [Inner; N] {
    type Item = T;
    type IntoPtrs = ChannelPtrsArray<T, N>;

    fn into_channel_ptrs(self) -> Self::IntoPtrs {
        let frames = self
            .iter()
            .map(Deref::deref)
            .map(<[T]>::len)
            .reduce(|a, b| {
                assert_eq!(a, b, "all channels must have equal length");
                a
            })
            .unwrap_or(0);
        Self::IntoPtrs {
            frames,
            channels: self.map(|c| c.as_ptr()),
        }
    }
}

// To avoid unintended allocations, this is never implicitly created.
// Invariant: All pointers point to `frames` initialized elements of type `T`.
#[cfg(feature = "alloc")]
pub struct ChannelPtrsBoxed<T> {
    frames: usize,
    channels: Box<[*const T]>,
}

#[cfg(feature = "alloc")]
// SAFETY: All pointers point to `self.frames` `T`s each.
unsafe impl<T> ChannelPtrs for ChannelPtrsBoxed<T> {
    type Item = T;

    fn frames(&self) -> usize {
        self.frames
    }

    fn channels(&self) -> u16 {
        self.channels.len().try_into().unwrap()
    }

    fn as_ptr(&self) -> *const *const Self::Item {
        self.channels.as_ptr()
    }
}

#[cfg(feature = "alloc")]
impl<T> ChannelPtrsBoxed<T> {
    pub fn from_slice<Inner: Deref<Target = [T]>>(slice: &[Inner]) -> Self {
        let frames = slice
            .iter()
            .map(Deref::deref)
            .map(<[T]>::len)
            .reduce(|a, b| {
                assert_eq!(a, b, "all channels must have equal length");
                a
            })
            .unwrap_or(0);
        let v: Vec<_> = slice.iter().map(|s| s.as_ptr()).collect();
        Self {
            frames,
            channels: v.into_boxed_slice(),
        }
    }

    // TODO: new(), with_capacity() -> switch from Box to Vec?

    // TODO: re-assign slice (with different length?)

    // TODO: try to re-assign slice with different lifetime (see https://github.com/mgeier/rsor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "alloc")]
    use alloc::vec;

    pub fn process(signal: impl IntoChannelPtrs<Item = f32>) {
        let ptrs = signal.into_channel_ptrs();
        let _ptr = ptrs.as_ptr();
        // This "pointer to pointers" would typically be passed to some C API.
        let _frames = ptrs.frames();
        let _channels = ptrs.channels();
    }

    #[test]
    fn from_array() {
        let a: [&[_]; _] = [&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]];
        process(a);
        #[cfg(feature = "alloc")]
        process([vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]);
    }

    #[test]
    fn from_slice() {
        let s: &[&[_]] = &[&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]];
        process(s);
        #[cfg(feature = "alloc")]
        process(vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]);
    }

    // Mono signals can be put into a one-element array.
    #[test]
    fn from_single_channel() {
        let mono: &[_] = &[1.0, 2.0, 3.0, 4.0];
        process([mono]);
        #[cfg(feature = "alloc")]
        let mono = vec![1.0, 2.0, 3.0, 4.0];
        process([mono]);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn boxed() {
        let s: &[&[_]] = &[
            &[1.0, 2.0, 3.0],
            &[1.0, 2.0, 3.0],
            &[1.0, 2.0, 3.0],
            &[1.0, 2.0, 3.0],
            &[1.0, 2.0, 3.0],
            &[1.0, 2.0, 3.0],
            &[1.0, 2.0, 3.0],
            &[1.0, 2.0, 3.0],
            &[1.0, 2.0, 3.0],
            &[1.0, 2.0, 3.0],
            &[1.0, 2.0, 3.0],
            &[1.0, 2.0, 3.0],
            &[1.0, 2.0, 3.0],
            &[1.0, 2.0, 3.0],
            &[1.0, 2.0, 3.0],
            &[1.0, 2.0, 3.0],
            &[1.0, 2.0, 3.0],
            &[1.0, 2.0, 3.0],
            &[1.0, 2.0, 3.0],
            &[1.0, 2.0, 3.0],
        ];
        let ptrs = ChannelPtrsBoxed::from_slice(s);
        process(ptrs);
    }
}
