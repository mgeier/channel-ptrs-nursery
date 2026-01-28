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
/// Conversion into [`ChannelPtrs`].
///
/// # Safety
///
/// The conversion must establish the safety guarantees of [`ChannelPtrs`].
pub unsafe trait IntoChannelPtrs {
    type Item;
    type IntoPtrs: ChannelPtrs<Item = Self::Item>;

    fn into_channel_ptrs(self) -> Self::IntoPtrs;
}

/// Blanket implementation for types that already implement the [`ChannelPtrs`] trait.
// SAFETY: All types that implement the ChannelPtrs trait fulfill the safety requirements.
unsafe impl<P: ChannelPtrs> IntoChannelPtrs for P {
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

// SAFETY: The implementation establishes the requirements of ChannelPtrs.
unsafe impl<T, Inner: Deref<Target = [T]>> IntoChannelPtrs for &[Inner] {
    type Item = T;

    type IntoPtrs = ChannelPtrsPartialArray<T, MAX_CHANNELS_FROM_SLICE>;

    fn into_channel_ptrs(self) -> Self::IntoPtrs {
        let frames = self
            .iter()
            .map(Deref::deref)
            .map(<[T]>::len)
            .reduce(|a, b| {
                assert_eq!(a, b, "all channels must have the same length");
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
// SAFETY: The implementation establishes the requirements of ChannelPtrs.
#[cfg(feature = "alloc")]
unsafe impl<T, Inner: Deref<Target = [T]>> IntoChannelPtrs for Vec<Inner> {
    type Item = T;

    type IntoPtrs = ChannelPtrsPartialArray<T, MAX_CHANNELS_FROM_SLICE>;

    fn into_channel_ptrs(self) -> Self::IntoPtrs {
        self.deref().into_channel_ptrs()
    }
}

// TODO: impl for boxed slice Box<Inner>

// SAFETY: The implementation establishes the requirements of ChannelPtrs.
unsafe impl<T, Inner: Deref<Target = [T]>, const N: usize> IntoChannelPtrs for [Inner; N] {
    type Item = T;
    type IntoPtrs = ChannelPtrsArray<T, N>;

    fn into_channel_ptrs(self) -> Self::IntoPtrs {
        let frames = self
            .iter()
            .map(Deref::deref)
            .map(<[T]>::len)
            .reduce(|a, b| {
                assert_eq!(a, b, "all channels must have the same length");
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
                assert_eq!(a, b, "all channels must have the same length");
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

// TODO: errors instead of panics?

pub fn channel_ptrs_from_slices_mut<T, Channel, Channels>(
    signal: Channels,
    storage: &mut [*mut T],
) -> (*mut *mut T, usize, u16)
where
    Channel: AsMut<[T]>,
    Channels: IntoIterator<Item = Channel>,
{
    let mut signal = signal.into_iter();
    let mut frames = None;
    let channels = signal
        .by_ref()
        .zip(storage.iter_mut())
        .map(|(mut ch, ptr)| {
            //channels += 1;
            let ch = ch.as_mut();
            let current_frames = ch.len();
            if let Some(f) = frames {
                assert_eq!(current_frames, f, "all channels must have the same length");
            } else {
                frames = Some(current_frames);
            }
            *ptr = ch.as_mut_ptr();
        })
        .count()
        .try_into()
        .expect("too many channels");
    assert!(signal.next().is_none(), "too few pointers in `storage`");
    (storage.as_mut_ptr(), frames.unwrap_or(0), channels)
}

pub unsafe fn channel_ptrs_to_slices_mut<'a, 'b, T>(
    ptrs: *mut *mut T,
    frames: usize,
    channels: u16,
) -> &'a mut [&'b mut [T]] {
    todo!()
}

// TODO: move to tests (or examples?)

#[derive(Default)]
pub struct Processor {
    channel_ptrs: [*mut f32; 6],
}

unsafe extern "C" fn do_nothing(_: *mut *mut f32, _: usize, _: u16) {}

impl Processor {
    // NB: This takes a mutable reference because it is *not* reentrant.
    // TODO: explain lifetimes ('b could be longer than 'a)
    pub fn process<'a, 'b, Channel, Channels>(&'a mut self, signal: Channels) -> &'a mut [&'b mut [f32]]
    where
        Channel: AsMut<[f32]> + 'b,
        Channels: IntoIterator<Item = Channel>,
    {
        let (ptrs, frames, channels) = channel_ptrs_from_slices_mut(signal, &mut self.channel_ptrs);

        // Let's pretend that we are passing `ptrs` to some FFI function here,
        // where the channel data will be overwritten.
        // SAFETY: Doing nothing is safe.
        unsafe {
            do_nothing(ptrs, frames, channels);
        }

        // SAFETY: Results from `channel_ptrs_from_slices_mut()` are valid for the given lifetimes.
        unsafe { channel_ptrs_to_slices_mut(ptrs, frames, channels) }
    }
}

pub fn process_iter<Channel, Channels>(signal: Channels) -> usize
where
    Channel: AsMut<[f32]>,
    Channels: IntoIterator<Item = Channel>,
{
    let mut channels = 0;
    let mut frames = None;
    let signal = signal.into_iter();
    for mut ch in signal {
        channels += 1;
        let current_frames = ch.as_mut().len();
        if let Some(f) = frames {
            assert_eq!(current_frames, f, "all channels must have the same length");
        } else {
            frames = Some(current_frames);
        }
        //ch.as_mut()[0] = 99.0;
    }
    channels
}

// TODO: copy_to_interleaved_uninit()

pub fn copy_to_interleaved<T, Channel, Channels>(source: Channels, destination: &mut [T])
where
    T: Copy,
    Channel: AsRef<[T]>,
    Channels: IntoIterator<IntoIter: ExactSizeIterator, Item = Channel>,
{
    let source = source.into_iter();
    let mut frames = None;
    // TODO: get channels from dest_len / frames and avoid ExactSizeIterator?
    // TODO: check if there are too many or too few channels
    let channels = source.len();
    for (offset, ch) in source.enumerate() {
        let ch = ch.as_ref();
        let current_frames = ch.len();
        if let Some(f) = frames {
            assert_eq!(current_frames, f, "all channels must have the same length");
        } else {
            // TODO: better error message?
            assert_eq!(
                current_frames * channels,
                destination.len(),
                "length mismatch"
            );
            frames = Some(current_frames);
        }
        for (dst, src) in destination
            .iter_mut()
            .skip(offset)
            .step_by(channels)
            .zip(ch)
        {
            *dst = *src;
        }
    }
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
    fn iter_from_slice() {
        let signal: &mut [&mut [_]] = &mut [&mut [1.0, 2.0, 3.0], &mut [4.0, 5.0, 6.0]];
        assert_eq!(process_iter(signal), 2);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn iter_from_vec() {
        let signal = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];
        assert_eq!(process_iter(signal), 2);
    }

    #[test]
    fn iter_from_array() {
        let signal: [&mut [_]; _] = [&mut [1.0, 2.0, 3.0], &mut [4.0, 5.0, 6.0]];
        assert_eq!(process_iter(signal), 2);
    }

    #[test]
    fn iter_from_chunks() {
        let back_to_back: &mut [_] = &mut [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        assert_eq!(process_iter(back_to_back.chunks_mut(3)), 2);
    }

    #[test]
    fn test_copy_to_interleaved() {
        let source: [&mut [_]; _] = [&mut [1.0, 2.0, 3.0], &mut [4.0, 5.0, 6.0]];
        let mut destination = [0.0; 6];
        copy_to_interleaved(source, &mut destination);
        assert_eq!(destination, [1.0, 4.0, 2.0, 5.0, 3.0, 6.0]);
    }

    #[test]
    fn process_slice() {
        let signal: &mut [&mut [_]] = &mut [&mut [1.0, 2.0, 3.0], &mut [4.0, 5.0, 6.0]];
        {
            let mut p = Processor::default();
            let result = p.process(signal);
            assert_eq!(result, [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
        }
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
