//! Pointers to channels (and more?).
//!
//! These are needed in some C APIs, do not use this in pure Rust code!

#![no_std]
#![forbid(clippy::undocumented_unsafe_blocks)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "ndarray")]
pub mod ndarray;

/*
#[cfg(doc)]
pub mod tutorial;
*/

// TODO: iter to slice? mention in ndarray module docs

// TODO: move this to example code:
/*
// This can be chosen arbitrarily as trade-off between stack usage and convenience.
const MAX_CHANNELS_FROM_SLICE: usize = 16;
*/

/// Creates channel pointers from a sequence of non-mutable slices.
///
/// If you have a single slice with non-interleaved channels,
/// you can use [`slice::chunks()`] to turn it into an appropriate sequence of slices.
///
/// For a mutable version see [`channel_ptrs_from_slices_mut()`].
///
/// # Panics
///
/// If the requested number of channels doesn't fit into `u16`.
pub fn channel_ptrs_from_slices<T, Channels>(
    signal: Channels,
    storage: &mut [*const T],
) -> Result<(*const *const T, usize, u16), Error>
where
    Channels: IntoIterator<Item: AsRef<[T]>>,
{
    let mut signal = signal.into_iter();
    let mut frames = None;
    let channels = signal
        .by_ref()
        .zip(storage.iter_mut())
        .try_fold(0usize, |acc, (ch, ptr)| {
            let ch = ch.as_ref();
            let current_frames = ch.len();
            if let Some(f) = frames {
                if current_frames != f {
                    return Err(Error::Jagged);
                }
            } else {
                frames = Some(current_frames);
            }
            *ptr = ch.as_ptr();
            Ok(acc + 1)
        })?
        .try_into()
        .expect("too many channels");
    if signal.next().is_some() {
        return Err(Error::StorageOverflow);
    }
    Ok((storage.as_ptr(), frames.unwrap_or(0), channels))
}

/// Creates channel pointers from a sequence of mutable slices.
///
/// If you have a single slice with non-interleaved channels,
/// you can use [`slice::chunks_mut()`] to turn it into an appropriate sequence of slices.
///
/// For a non-mutable version see [`channel_ptrs_from_slices()`].
///
/// # Panics
///
/// If the requested number of channels doesn't fit into `u16`.
pub fn channel_ptrs_from_slices_mut<T, Channels>(
    signal: Channels,
    storage: &mut [*mut T],
) -> Result<(*mut *mut T, usize, u16), Error>
where
    Channels: IntoIterator<Item: AsMut<[T]>>,
{
    let mut signal = signal.into_iter();
    let mut frames = None;
    let channels = signal
        .by_ref()
        .zip(storage.iter_mut())
        .try_fold(0usize, |acc, (mut ch, ptr)| {
            let ch = ch.as_mut();
            let current_frames = ch.len();
            if let Some(f) = frames {
                if current_frames != f {
                    return Err(Error::Jagged);
                }
            } else {
                frames = Some(current_frames);
            }
            *ptr = ch.as_mut_ptr();
            Ok(acc + 1)
        })?
        .try_into()
        .expect("too many channels");
    if signal.next().is_some() {
        return Err(Error::StorageOverflow);
    }
    Ok((storage.as_mut_ptr(), frames.unwrap_or(0), channels))
}

// TODO: channel pointers from uninit slices?

/// Creates a non-mutable slice of slices from channel pointers.
///
/// In most cases, [`channel_ptrs_to_slices()`] is easier to use and should be preferred.
///
/// For a mutable version see [`channel_ptrs_to_nested_slices_mut()`].
///
/// # Safety
///
/// TODO: many things
///
/// TODO: memory must be initialized. add uninit variant?
pub unsafe fn channel_ptrs_to_nested_slices<'b, T>(
    ptrs: *const *const T,
    frames: usize,
    channels: u16,
    storage: &mut [*const [T]],
) -> Result<&[&'b [T]], Error> {
    let channels = channels.into();
    if storage.len() < channels {
        return Err(Error::StorageOverflow);
    }
    for (i, channel_slice) in storage.iter_mut().enumerate().take(channels) {
        // SAFETY: Caller must ensure requirements stated in docstring.
        let s = unsafe { core::slice::from_raw_parts(*ptrs.add(i), frames) };
        *channel_slice = s;
    }
    // SAFETY: The correct number of slices has been initialized above.
    Ok(unsafe { core::slice::from_raw_parts(storage.as_ptr() as *const &[_], channels) })
}

/// Creates a mutable slice of slices from channel pointers.
///
/// In most cases, [`channel_ptrs_to_slices_mut()`] is easier to use and should be preferred.
///
/// For a non-mutable version see [`channel_ptrs_to_nested_slices()`].
///
/// # Safety
///
/// TODO: many things, refer to channel_ptrs_to_nested_slices()?
pub unsafe fn channel_ptrs_to_nested_slices_mut<'b, T>(
    ptrs: *mut *mut T,
    frames: usize,
    channels: u16,
    storage: &mut [*mut [T]],
) -> Result<&mut [&'b mut [T]], Error> {
    let channels = channels.into();
    if storage.len() < channels {
        return Err(Error::StorageOverflow);
    }
    for (i, channel_slice) in storage.iter_mut().enumerate().take(channels) {
        // SAFETY: Caller must ensure requirements stated in docstring.
        let s = unsafe { core::slice::from_raw_parts_mut(*ptrs.add(i), frames) };
        *channel_slice = s;
    }
    // SAFETY: The correct number of slices has been initialized above.
    Ok(unsafe { core::slice::from_raw_parts_mut(storage.as_mut_ptr() as *mut &mut [_], channels) })
}

/// Creates an iterator over non-mutable slices from channel pointers.
///
/// If possible, this should be preferred over [`channel_ptrs_to_nested_slices()`],
/// because it doesn't need any extra `storage`.
///
/// For a mutable version see [`channel_ptrs_to_slices_mut()`].
///
/// # Safety
///
/// TODO: many things
pub unsafe fn channel_ptrs_to_slices<'b, T: 'b>(
    ptrs: *const *const T,
    frames: usize,
    channels: u16,
) -> impl Iterator<Item = &'b [T]> {
    (0..usize::from(channels)).map(move |i| {
        // SAFETY: Caller must ensure requirements stated in docstring.
        unsafe { core::slice::from_raw_parts(*ptrs.add(i), frames) }
    })
}

/// Creates an iterator over mutable slices from channel pointers.
///
/// If possible, this should be preferred over [`channel_ptrs_to_nested_slices_mut()`],
/// because it doesn't need any extra `storage`.
///
/// For a mutable version see [`channel_ptrs_to_slices()`].
///
/// # Safety
///
/// TODO: many things
pub unsafe fn channel_ptrs_to_slices_mut<'b, T: 'b>(
    ptrs: *mut *mut T,
    frames: usize,
    channels: u16,
) -> impl Iterator<Item = &'b mut [T]> {
    (0..usize::from(channels)).map(move |i| {
        // SAFETY: Caller must ensure requirements stated in docstring.
        unsafe { core::slice::from_raw_parts_mut(*ptrs.add(i), frames) }
    })
}

// TODO: move to tests (or examples?)

pub struct Processor {
    channel_ptrs: [*mut f32; 6],
    //channel_refs: [MaybeUninit<&'static mut [f32]>; 6],
    //channel_refs: [*mut [f32]; 6],
}

impl Processor {
    pub fn new() -> Self {
        Self {
            channel_ptrs: [core::ptr::null_mut(); _],
            //channel_refs: [const { MaybeUninit::uninit() }; _],
            // https://github.com/rust-lang/rust/issues/66316
            //channel_refs: [core::ptr::null_mut::<[f32; 0]>() as *mut [f32]; _],
        }
    }
}

impl Default for Processor {
    fn default() -> Self {
        Self::new()
    }
}

// This is a stand-in for some FFI function.
unsafe extern "C" fn set_a_value(ptrs: *mut *mut f32, frames: usize, channels: u16) {
    assert!(0 < frames && 0 < channels);
    // SAFETY: The pointer is valid and there is at least one frame and one channel.
    unsafe {
        (*ptrs).write(99.9);
    }
}

impl Processor {
    // NB: This takes a mutable reference to `self` because it is *not* reentrant.
    pub fn process<'c, Channel, Channels>(&mut self, signal: Channels)
    where
        Channels: IntoIterator<Item = &'c mut Channel>,
        Channel: AsMut<[f32]> + ?Sized + 'c,
    {
        let (ptrs, frames, channels) =
            channel_ptrs_from_slices_mut(signal, &mut self.channel_ptrs).unwrap();

        // SAFETY: channel_ptrs_from_slices_mut() returned valid results.
        unsafe {
            set_a_value(ptrs, frames, channels);
        }
    }
}

// TODO: copy_to_interleaved_uninit()

// TODO: copy_from_interleaved()? ..._uninit()?

// TODO: copy_to_noninterleaved()? use memcpy?

// TODO: noninterleaved_to_* via chunks()/chunks_mut()?

pub fn copy_to_interleaved<T, Channels>(
    source: Channels,
    destination: &mut [T],
) -> Result<(), Error>
where
    T: Copy,
    Channels: IntoIterator<IntoIter: ExactSizeIterator, Item: AsRef<[T]>>,
{
    let source = source.into_iter();
    let mut frames = None;
    // TODO: get channels from dest_len / frames and avoid ExactSizeIterator?
    // TODO: mention ExactSizeIterator in the docs nevertheless
    // TODO: check if there are too many or too few channels
    let channels = source.len();
    for (offset, ch) in source.enumerate() {
        let ch = ch.as_ref();
        let current_frames = ch.len();
        if let Some(f) = frames {
            if current_frames != f {
                return Err(Error::Jagged);
            }
        } else {
            if current_frames * channels != destination.len() {
                return Err(Error::LengthMismatch);
            }
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
    Ok(())
}

// TODO: multiple errors? rename?
#[derive(Debug)]
pub enum Error {
    // all channels must have the same length
    Jagged,
    // TODO: not all functions need this
    LengthMismatch,
    // TODO: not all functions need this
    // too few pointers in `storage`
    StorageOverflow,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "alloc")]
    use alloc::vec;

    #[test]
    fn test_copy_to_interleaved() {
        let source: [&mut [_]; _] = [&mut [1.0, 2.0, 3.0], &mut [4.0, 5.0, 6.0]];
        let mut destination = [0.0; 6];
        copy_to_interleaved(source, &mut destination).unwrap();
        assert_eq!(destination, [1.0, 4.0, 2.0, 5.0, 3.0, 6.0]);
    }

    #[test]
    fn process_slice() {
        let ch0 = [1.0, 2.0, 3.0];
        let ch1 = [4.0, 5.0, 6.0];
        // TODO: array is `Copy` so this copies each channel and modifies the copy!
        let signal: &mut [_] = &mut [ch0, ch1];
        let mut p = Processor::new();
        p.process(signal);
        assert_eq!(ch0, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn process_array() {
        let mut p = Processor::new();

        let mut ch0 = [1.0, 2.0, 3.0];
        let mut ch1 = [4.0, 5.0, 6.0];
        p.process([&mut ch0, &mut ch1]);
        assert_eq!(ch0, [99.9, 2.0, 3.0]);

        let mut signal = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
        p.process(&mut signal);
        assert_eq!(signal, [[99.9, 2.0, 3.0], [4.0, 5.0, 6.0]]);

        #[cfg(feature = "alloc")]
        {
            let mut ch0 = vec![1.0, 2.0, 3.0];
            let mut ch1 = vec![4.0, 5.0, 6.0];
            p.process([&mut ch0, &mut ch1]);
            assert_eq!(ch0, [99.9, 2.0, 3.0]);
        }
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn process_vec() {
        let mut signal = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];
        let mut p = Processor::new();
        p.process(&mut signal);
        assert_eq!(signal, [[99.9, 2.0, 3.0], [4.0, 5.0, 6.0]]);
    }

    // stacked == non-interleaved
    #[test]
    fn process_stacked() {
        let mut data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let mut p = Processor::new();
        p.process(data.chunks_mut(3));
        assert_eq!(data, [99.9, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    // Mono signals can be put into a one-element array.
    #[test]
    fn process_single_channel() {
        let mut mono = [1.0, 2.0, 3.0, 4.0];
        let mut p = Processor::new();
        p.process([&mut mono]);
        assert_eq!(mono, [99.9, 2.0, 3.0, 4.0]);
        #[cfg(feature = "alloc")]
        let mut mono = vec![1.0, 2.0, 3.0, 4.0];
        p.process([&mut mono]);
        assert_eq!(mono, [99.9, 2.0, 3.0, 4.0]);
    }
}
