#![allow(unused)]

use core::marker::PhantomData;

pub struct MultiSlice<'a, T> {
    frames: usize,
    channels: Box<[*const T]>,
    _marker: PhantomData<&'a ()>,
}

pub struct MultiSliceConst<T, const N: usize> {
    frames: usize,
    channels: [*const T; N],
}

// TODO: partially used array?

// TODO: panic or alloc on overflow?

// TODO: never alloc by default, but provide separate explicit type for arbitrary channel numbers

pub struct MultiSliceAlloc<'a, T> {
    frames: usize,
    channels: Box<[*const T]>,
    _marker: PhantomData<&'a ()>,
}

// Modeled after IntoIterator.
pub trait IntoChannelPtrs {
    type Item;
    type IntoMulti: ChannelPtrs<Item = Self::Item>;

    fn into_multi_slice(self) -> Self::IntoMulti;
}

pub trait ChannelPtrs {
    type Item;

    // TODO: ???
}

impl<T, R: AsRef<[T]>> From<&[R]> for MultiSlice<'_, T> {
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
        MultiSlice {
            frames,
            channels: v.into_boxed_slice(),
            _marker: PhantomData,
        }
    }
}

impl<T, R: AsRef<[T]>, const N: usize> From<[R; N]> for MultiSliceConst<T, N> {
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
        MultiSliceConst {
            frames,
            channels: channels.map(|c| c.as_ref().as_ptr()),
        }
    }
}

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

impl<T, const N: usize> MultiSliceConst<T, N> {
    pub fn len(&self) -> usize {
        self.frames
    }
}

fn array_of_vec2array_of_ptr<T, const N: usize>(a: [Vec<T>; N]) -> [*const T; N] {
    a.map(|v| v.as_ptr())
}

//pub fn process(signal: impl IntoChannelPtrs<Item = f32>) {}
pub fn process<C: ChannelPtrs>(signal: impl Into<C>) {}
//pub fn process(signal: impl Into<impl ChannelPtrs>) {}

impl<T, const N: usize> ChannelPtrs for MultiSliceConst<T, N> {
    type Item = T;
}

#[cfg(test)]
mod tests {
    use super::*;

    /*
    // TODO: this should not compile (dangling reference)
    fn return_slice<'a>() -> MultiSlice<'a, i32> {
        let v = [vec![1, 2, 3], vec![4, 5, 6]];
        let p = array_of_vec2array_of_ptr(v);
        MultiSlice::from_ptrs(p, 3)
    }
    */

    #[test]
    fn from_array() {
        let a = [&[1, 2, 3][..], &[4, 5, 6][..]];
        process::<MultiSliceConst<_, _>>(a);
        //process(a);
    }

    #[test]
    fn basic() {
        let a = &[&[1, 2, 3][..], &[4, 5, 6][..]][..];
        let _s = MultiSlice::from(a);

        /*
        let v = [vec![1, 2, 3], vec![4, 5, 6]];
        let p = array_of_vec2array_of_ptr(v);
        let _s = MultiSlice::from_ptrs(p, 3);
        */

        /*
        let _x = return_slice();
        */
    }

    #[test]
    fn const_slice() {
        let v = [vec![1, 2, 3], vec![4, 5, 6]];
        let s = MultiSliceConst::from(v);
        assert_eq!(s.len(), 3);
        let a = [&[1, 2, 3][..], &[4, 5, 6][..]];
        let s = MultiSliceConst::from(a);
        assert_eq!(s.len(), 3);
    }
}
