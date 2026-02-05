//! Pointers to channels.
//!
//! These are needed in some C APIs, do not use this in pure Rust code!

#![no_std]
#![forbid(clippy::undocumented_unsafe_blocks)]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::mem::MaybeUninit;

// TODO: move this to example code:
/*
// This can be chosen arbitrarily as trade-off between stack usage and convenience.
const MAX_CHANNELS_FROM_SLICE: usize = 16;
*/

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

/// Slices from pointers ...
///
/// # Safety
///
/// TODO: many things
pub unsafe fn channel_ptrs_to_slices_mut<'a, 'b, T>(
    ptrs: *mut *mut T,
    frames: usize,
    channels: u16,
    storage: &'a mut [MaybeUninit<&'b mut [T]>],
) -> &'a mut [&'b mut [T]] {
    let channels = channels.into();
    assert!(channels <= storage.len(), "not enough space in `storage`");
    for (i, channel_slice) in storage.iter_mut().enumerate().take(channels) {
        // SAFETY: Caller must ensure requirements stated in docstring.
        let s = unsafe { core::slice::from_raw_parts_mut(*ptrs.add(i), frames) };
        *channel_slice = MaybeUninit::new(s);
    }
    // SAFETY: The correct number of slices has been initialized above.
    unsafe { core::slice::from_raw_parts_mut(storage.as_mut_ptr() as *mut &mut [_], channels) }
}

// TODO: channel_ptrs_to_iterator (without extra storage?)

// TODO: move to tests (or examples?)

pub struct Processor {
    channel_ptrs: [*mut f32; 6],
    channel_refs: [MaybeUninit<&'static mut [f32]>; 6],
}

impl Processor {
    pub fn new() -> Self {
        Self {
            channel_ptrs: [core::ptr::null_mut(); _],
            channel_refs: [const { MaybeUninit::uninit() }; _],
        }
    }
}

unsafe extern "C" fn do_nothing(_: *mut *mut f32, _: usize, _: u16) {}

impl Processor {
    // NB: This takes a mutable reference because it is *not* reentrant.
    // TODO: explain lifetimes ('b could be longer than 'a)
    // Using two lifetimes here allows for maximum flexibility.
    // In most situations, one lifetime would work just as well
    // (and the lifetime notation could be elided)
    pub fn process<'a, 'b, Channel, Channels>(
        &'a mut self,
        signal: Channels,
    ) -> &'a mut [&'b mut [f32]]
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
        unsafe { channel_ptrs_to_slices_mut(ptrs, frames, channels, &mut self.channel_refs) }
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
        let channel0;
        {
            let mut p = Processor::new();
            let result = p.process(signal);
            assert_eq!(result, [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
            channel0 = core::mem::take(&mut result[0]);
        }
        // The lifetime 'a of the outer slice (stored in the Processor) has already ended,
        // but the inner slice with lifetime 'b is still alive.
        assert_eq!(channel0, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn from_array() {
        let signal: [&mut [_]; _] = [&mut [1.0, 2.0, 3.0], &mut [4.0, 5.0, 6.0]];
        let mut p = Processor::new();
        let result = p.process(signal);
        assert_eq!(result, [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
        #[cfg(feature = "alloc")]
        {
            let result = p.process([vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]);
            assert_eq!(result, [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
        }
    }

    #[test]
    fn from_slice() {
        let signal: &mut [&mut [_]] = &mut [&mut [1.0, 2.0, 3.0], &mut [4.0, 5.0, 6.0]];
        let mut p = Processor::new();
        let result = p.process(signal);
        assert_eq!(result, [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);

        #[cfg(feature = "alloc")]
        {
            let result = p.process(vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]);
            assert_eq!(result, [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
        }
    }

    // Mono signals can be put into a one-element array.
    #[test]
    fn from_single_channel() {
        let mono: &mut [_] = &mut [1.0, 2.0, 3.0, 4.0];
        let mut p = Processor::new();
        let result = p.process([mono]);
        assert_eq!(result, [[1.0, 2.0, 3.0, 4.0]]);
        #[cfg(feature = "alloc")]
        let mono = vec![1.0, 2.0, 3.0, 4.0];
        let result = p.process([mono]);
        assert_eq!(result, [[1.0, 2.0, 3.0, 4.0]]);
    }

    /*
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
        let mut storage = [core::ptr::null_mut(); 6];
        let (ptrs, frames, channels) = channel_ptrs_from_slices(s, &mut storage);
        unsafe { do_nothing(ptrs, frames, channels) };
    }
    */
}
