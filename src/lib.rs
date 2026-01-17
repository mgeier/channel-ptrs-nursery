#![no_std]
#![allow(unused)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

// TODO: remove Vec dependency
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use core::marker::PhantomData;

pub struct ChannelPtrs<'a, T, const N: usize = 2> {
    frames: usize,
    channels: Storage<T, N>,
    _marker: PhantomData<&'a ()>,
}

enum Storage<T, const N: usize> {
    // TODO: check whether that's also true for 32 bit systems:
    // NB: We don't strictly need separate Array and PartialArray,
    // but it doesn't need more space, so why not make it more explicit?
    Array([*const T; N]),
    // NB: using u32 would lead to the same size on the stack,
    // but u16 probably makes sense for channel counts.
    PartialArray(u16, [*const T; N]),
    #[cfg(feature = "alloc")]
    Boxed(Box<[*const T]>),
}

// TODO: panic or alloc on overflow?

// TODO: never alloc by default, but provide separate explicit type for arbitrary channel numbers

/*
// This is only an option if allocations should be allowed to happen implicitly!
impl<T, R: AsRef<[T]>> From<&[R]> for ChannelPtrs<'_, T> {
    fn from(slice: &[R]) -> Self {
        let frames = slice
            .iter()
            .map(AsRef::as_ref)
            .map(<[T]>::len)
            .reduce(|a, b| {
                assert_eq!(a, b, "all channels must have equal length");
                a
            })
            .unwrap_or(0);
        let v: Vec<_> = slice.iter().map(|s| s.as_ref().as_ptr()).collect();
        Self {
            frames,
            channels: Storage::Boxed(v.into_boxed_slice()),
            _marker: PhantomData,
        }
    }
}
*/

// This can be chosen arbitrary as trade-off between stack usage and convenience.
const MAX_CHANNELS_FROM_SLICE: usize = 16;

impl<T, R: AsRef<[T]>> From<&[R]> for ChannelPtrs<'_, T, MAX_CHANNELS_FROM_SLICE> {
    fn from(slice: &[R]) -> Self {
        let frames = slice
            .iter()
            .map(AsRef::as_ref)
            .map(<[T]>::len)
            .reduce(|a, b| {
                assert_eq!(a, b, "all channels must have equal length");
                a
            })
            .unwrap_or(0);
        let channels: u16 = slice.len().try_into().expect("slice too long");
        if usize::from(channels) > MAX_CHANNELS_FROM_SLICE {
            panic!("Too many channels for automatic conversion: {channels} \
                (maximum: {MAX_CHANNELS_FROM_SLICE})\nUse ChannelPtrs::TODO() instead.");
        }
        let mut ptrs = [core::ptr::dangling(); MAX_CHANNELS_FROM_SLICE];
        // NB: zip() stops when one of the iterators is exhausted.
        for (src, dst) in slice.iter().map(AsRef::as_ref).zip(ptrs.iter_mut()) {
            *dst = src.as_ptr();
        }
        Self {
            frames,
            channels: Storage::PartialArray(channels, ptrs),
            _marker: PhantomData,
        }
    }
}

/*
impl<T> From<&[T]> for ChannelPtrs<'_, T, 1> {
    fn from(slice: &[T]) -> Self {
        Self {
            frames: slice.len(),
            channels: Storage::Array([slice.as_ptr()]),
            _marker: PhantomData,
        }
    }
}
*/

impl<T, R: AsRef<[T]>, const N: usize> From<[R; N]> for ChannelPtrs<'_, T, N> {
    fn from(channels: [R; N]) -> Self {
        let frames = channels
            .iter()
            .map(AsRef::as_ref)
            .map(<[T]>::len)
            .reduce(|a, b| {
                assert_eq!(a, b, "all channels must have equal length");
                a
            })
            .unwrap_or(0);
        Self {
            frames,
            channels: Storage::Array(channels.map(|c| c.as_ref().as_ptr())),
            _marker: PhantomData,
        }
    }
}

// TODO: from array of ptrs vs. slice of ptrs
// TODO: does this have to be unsafe because lifetime cannot be checked?
// TODO: allow implicit conversion instead?
/*
impl<T> MultiSlice<'_, T> {
    fn from_ptrs<const N: usize>(ptrs: [*const T; N], frames: usize) -> Self {
        MultiSlice {
            frames,
            channels: Channels::Borrowed((ptrs.as_ptr(), N)),
            _marker: PhantomData,
        }
    }

    pub fn as_ptrs(&self) -> *const *const T {
        match &self.channels {
            Channels::Owned(b) => b.as_ptr(),
            Channels::Borrowed((ptrs, _channels)) => *ptrs,
        }
    }
}
*/

impl<T, const N: usize> ChannelPtrs<'_, T, N> {
    pub fn frames(&self) -> usize {
        self.frames
    }
}

fn array_of_vec2array_of_ptr<T, const N: usize>(a: [Vec<T>; N]) -> [*const T; N] {
    a.map(|v| v.as_ptr())
}

pub fn process<'a, const N: usize>(signal: impl Into<ChannelPtrs<'a, f32, N>>) {}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::vec;

    /*
    // TODO: this should not compile (dangling reference)
    fn return_slice<'a>() -> ChannelPtrs<'a, i32> {
        let v = [vec![1, 2, 3], vec![4, 5, 6]];
        let p = array_of_vec2array_of_ptr(v);
        ChannelPtrs::from_ptrs(p, 3)
    }
    */

    #[test]
    fn from_array() {
        let a = [&[1.0, 2.0, 3.0][..], &[4.0, 5.0, 6.0][..]];
        process(a);

        process([&vec![1.0, 2.0, 3.0], &vec![4.0, 5.0, 6.0]]);
    }

    #[test]
    fn from_slice() {
        /*
        let v = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];
        process(v);
        */
        let s = &[&[1.0, 2.0, 3.0][..], &[4.0, 5.0, 6.0][..]][..];
        process(s);
        // TODO: different lengths should cause an error!
        let ch0 = vec![1.0, 2.0, 3.0, 4.0];
        let ch1 = vec![4.0, 5.0, 6.0];
        let both = vec![ch0, ch1];
        process(both.as_ref());
    }

    /*
    #[test]
    fn from_single_channel() {
        let mono = vec![1.0, 2.0, 3.0, 4.0];
        process(mono.as_ref());
    }
    */

    #[test]
    fn size() {
        // 2 * usize minimum + 8 bytes discriminator
        // TODO: consider size of usize
        // TODO: consider disabled "alloc" feature
        assert_eq!(core::mem::size_of::<Storage<f32, 1>>(), 2 * 8 + 8);
        assert_eq!(core::mem::size_of::<Storage<f32, 2>>(), 2 * 8 + 8);
        assert_eq!(core::mem::size_of::<Storage<f32, 3>>(), 3 * 8 + 8);
        assert_eq!(core::mem::size_of::<Storage<f32, 4>>(), 4 * 8 + 8);
    }

    #[test]
    fn basic() {
        let a = &[&[1, 2, 3][..], &[4, 5, 6][..]][..];
        let _s = ChannelPtrs::from(a);

        /*
        let v = [vec![1, 2, 3], vec![4, 5, 6]];
        let p = array_of_vec2array_of_ptr(v);
        let _s = ChannelPtrs::from_ptrs(p, 3);
        */

        /*
        let _x = return_slice();
        */
    }

    #[test]
    fn from_array2() {
        let v = [vec![1, 2, 3], vec![4, 5, 6]];
        let s = ChannelPtrs::from(v);
        assert_eq!(s.frames(), 3);
        let a = [&[1, 2, 3][..], &[4, 5, 6][..]];
        let s = ChannelPtrs::from(a);
        assert_eq!(s.frames(), 3);
    }
}
