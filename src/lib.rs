//! Pointers to channels (and more?).
//!
//! These are needed in some C APIs, do not use this in pure Rust code!

#![no_std]
#![forbid(clippy::undocumented_unsafe_blocks)]

use core::mem::MaybeUninit;

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

/// A non-mutable audio channel in contiguous memory.
///
/// This can be used to define generic function arguments
/// (using `impl IntoIterator<Item: Channel<T>>`) that accept multi-channel signals.
///
/// # Examples
///
/// ```
/// use much::Channel;
///
/// fn process(channels: impl IntoIterator<Item: Channel<f32>>) {
///     // Use in for-loop or call .into_iter():
///     for channel in channels {
///         // Call .as_ref() on each channel to get a "normal" slice:
///         let channel: &[f32] = channel.as_ref();
///         assert_eq!(channel[0], 0.5);
///     }
/// }
///
/// // TODO: mention ExactSizeIterator?
///
/// // This function can be used in many different ways:
///
/// let a = [[0.5, 0.6, 0.7, 0.8], [0.5, 0.4, 0.3, 0.2]];
/// process(&a);
///
/// let v = vec![vec![0.5, 0.6, 0.7, 0.8], vec![0.5, 0.4, 0.3, 0.2]];
/// process(&v);
///
/// let left = [0.5, 0.6, 0.7, 0.8];
/// let right = [0.5, 0.4, 0.3, 0.2];
/// process([&left, &right]);
///
/// let noninterleaved = [0.5, 0.6, 0.7, 0.8, 0.5, 0.4, 0.3, 0.2];
/// process(noninterleaved.chunks(4));
/// ```
pub trait Channel<T>: AsRef<[T]> {}

impl<T, U: AsRef<[T]> + ?Sized> Channel<T> for &U {}

/// A mutable audio channel in contiguous memory.
///
/// This can be used to define generic function arguments
/// (using `impl IntoIterator<Item: ChannelMut<T>>`) that accept multi-channel signals.
///
/// # Examples
///
/// ```
/// use much::ChannelMut;
///
/// fn process(channels: impl IntoIterator<Item: ChannelMut<f32>>) {
///     // Use in for-loop or call .into_iter():
///     for mut channel in channels {
///         // Call .as_mut() on each channel to get a "normal" writable slice:
///         let channel: &mut [f32] = channel.as_mut();
///         channel[0] = 0.99;
///     }
/// }
///
/// // This function can be used in many different ways:
///
/// let mut a = [[0.5, 0.6, 0.7, 0.8], [0.5, 0.4, 0.3, 0.2]];
/// process(&mut a);
/// assert_eq!(a, [[0.99, 0.6, 0.7, 0.8], [0.99, 0.4, 0.3, 0.2]]);
///
/// let mut v = vec![vec![0.5, 0.6, 0.7, 0.8], vec![0.5, 0.4, 0.3, 0.2]];
/// process(&mut v);
/// assert_eq!(v, [[0.99, 0.6, 0.7, 0.8], [0.99, 0.4, 0.3, 0.2]]);
///
/// let mut left = [0.5, 0.6, 0.7, 0.8];
/// let mut right = [0.5, 0.4, 0.3, 0.2];
/// process([&mut left, &mut right]);
/// assert_eq!(left, [0.99, 0.6, 0.7, 0.8]);
/// assert_eq!(right, [0.99, 0.4, 0.3, 0.2]);
///
/// let mut noninterleaved = [0.5, 0.6, 0.7, 0.8, 0.5, 0.4, 0.3, 0.2];
/// process(noninterleaved.chunks_mut(4));
/// assert_eq!(noninterleaved, [0.99, 0.6, 0.7, 0.8, 0.99, 0.4, 0.3, 0.2]);
/// ```
pub trait ChannelMut<T>: AsMut<[T]> {}

impl<T, U: AsMut<[T]> + ?Sized> ChannelMut<T> for &mut U {}

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
pub fn channel_ptrs_from_slices<T>(
    signal: impl IntoIterator<Item: Channel<T>>,
    storage: &mut [*const T],
) -> Result<(*const *const T, usize, u16), Error> {
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
pub fn channel_ptrs_from_slices_mut<T>(
    signal: impl IntoIterator<Item: ChannelMut<T>>,
    storage: &mut [*mut T],
) -> Result<(*mut *mut T, usize, u16), Error> {
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

// TODO: do we need those? docs: users can create their own traits (+ blanket impls)
pub trait ChannelsMut<T>: IntoIterator<IntoIter: ExactSizeIterator, Item: ChannelMut<T>> {}
impl<T, U: IntoIterator<IntoIter: ExactSizeIterator, Item: ChannelMut<T>>> ChannelsMut<T> for U {}

impl Processor {
    // NB: This takes a mutable reference to `self` because it is *not* reentrant.
    pub fn process(&mut self, signal: impl IntoIterator<Item: ChannelMut<f32>>) {
        let (ptrs, frames, channels) =
            channel_ptrs_from_slices_mut(signal, &mut self.channel_ptrs).unwrap();

        // SAFETY: channel_ptrs_from_slices_mut() returned valid results.
        unsafe {
            set_a_value(ptrs, frames, channels);
        }
    }
}

/// Copies all samples from a single slice of interleaved channels into contiguous channels.
///
/// # Errors
///
/// [`Error::Jagged`] if not all destination channels have the same length.
/// [`Error::LengthMismatch`] if the samples don't fit snugly into the destination.
// TODO: refer to copy_to_interleaved for ExactSizeIterator etc.
pub fn copy_from_interleaved<T>(
    source: &mut [T],
    destination: impl IntoIterator<IntoIter: ExactSizeIterator, Item: ChannelMut<T>>,
) -> Result<(), Error>
where
    T: Copy,
{
    copy_from_interleaved_uninit(
        source,
        destination.into_iter().map(|mut ch| {
            let ch = ch.as_mut();
            // SAFETY: TODO: same as above?
            unsafe { core::slice::from_raw_parts_mut(ch.as_mut_ptr().cast(), ch.len()) }
        }),
    )
}

/// Copies all samples from a single slice of interleaved channels into contiguous uninitialized channels.
///
/// Same as [`copy_from_interleaved()`], but writing into uninitialized channels.
pub fn copy_from_interleaved_uninit<T>(
    source: &mut [T],
    destination: impl IntoIterator<IntoIter: ExactSizeIterator, Item: ChannelMut<MaybeUninit<T>>>,
) -> Result<(), Error>
where
    T: Copy,
{
    copy_from_interleaved_uninit_and_iterate(source, destination).try_for_each(|ch| ch.map(|_| ()))
}

/// Copies all samples from a single slice of interleaved channels into contiguous uninitialized
/// channels and returns an iterator over now-initialized channels.
///
/// Same as [`copy_from_interleaved_uninit()`], but returning an iterator.
pub fn copy_from_interleaved_uninit_and_iterate<T>(
    source: &mut [T],
    destination: impl IntoIterator<IntoIter: ExactSizeIterator, Item: ChannelMut<MaybeUninit<T>>>,
) -> impl ExactSizeIterator<Item = Result<&mut [T], Error>>
where
    T: Copy,
{
    let destination = destination.into_iter();
    let channels = destination.len();
    let mut frames = None;
    destination.enumerate().map(move |(i, mut ch)| {
        let ch = ch.as_mut();
        let current_frames = ch.len();
        if let Some(f) = frames {
            if current_frames != f {
                return Err(Error::Jagged);
            }
        } else {
            if current_frames * channels != source.len() {
                return Err(Error::LengthMismatch);
            }
            frames = Some(current_frames);
        }
        for (dst, src) in ch
            .iter_mut()
            .zip(source.iter_mut().skip(i).step_by(channels))
        {
            *dst = MaybeUninit::new(*src);
        }
        // SAFETY: TODO: see above?
        Ok(unsafe { core::slice::from_raw_parts_mut(ch.as_mut_ptr().cast(), ch.len()) })
    })
}

/// Copies all samples from a single slice of non-interleaved channels into separate channels.
///
/// This is likely faster than [`copy_from_interleaved()`] because contiguous chunks can be copied.
///
/// It is likely even faster to not copy the channels at all.
/// If a function accepts an iterator over (contiguous) channels,
/// [`source.chunks()`](slice::chunks) can be used to create an appropriate iterator
/// from a slice of non-interleaved channels.
///
/// # Errors
///
/// [`Error::Jagged`] if not all destination channels have the same length.
/// [`Error::LengthMismatch`] if the samples don't fit snugly into the destination.
pub fn copy_from_noninterleaved<T>(
    source: &mut [T],
    destination: impl IntoIterator<IntoIter: ExactSizeIterator, Item: ChannelMut<T>>,
) -> Result<(), Error>
where
    T: Copy,
{
    copy_from_noninterleaved_uninit(
        source,
        destination.into_iter().map(|mut ch| {
            let ch = ch.as_mut();
            // SAFETY: TODO: same as above?
            unsafe { core::slice::from_raw_parts_mut(ch.as_mut_ptr().cast(), ch.len()) }
        }),
    )
}

/// Copies all samples from a single slice of non-interleaved channels into separate uninitialized channels.
///
/// Same as [`copy_from_noninterleaved()`], but writing into uninitialized channels.
pub fn copy_from_noninterleaved_uninit<T>(
    source: &mut [T],
    destination: impl IntoIterator<IntoIter: ExactSizeIterator, Item: ChannelMut<MaybeUninit<T>>>,
) -> Result<(), Error>
where
    T: Copy,
{
    copy_from_noninterleaved_uninit_and_iterate(source, destination)
        .try_for_each(|ch| ch.map(|_| ()))
}

/// Copies all samples from a single slice of non-interleaved channels into separate uninitialized
/// channels and returns an iterator over now-initialized channels.
///
/// Same as [`copy_from_noninterleaved_uninit()`], but returning an iterator.
pub fn copy_from_noninterleaved_uninit_and_iterate<T>(
    source: &mut [T],
    destination: impl IntoIterator<IntoIter: ExactSizeIterator, Item: ChannelMut<MaybeUninit<T>>>,
) -> impl ExactSizeIterator<Item = Result<&mut [T], Error>>
where
    T: Copy,
{
    let destination = destination.into_iter();
    let channels = destination.len();
    let mut frames = None;
    destination.enumerate().map(move |(i, mut ch)| {
        let ch = ch.as_mut();
        let current_frames = ch.len();
        if let Some(f) = frames {
            if current_frames != f {
                return Err(Error::Jagged);
            }
        } else {
            if current_frames * channels != source.len() {
                return Err(Error::LengthMismatch);
            }
            frames = Some(current_frames);
        }
        // SAFETY: Source and destination point to the right amount of elements
        // and non-overlapping uninitialized space, respectively.
        unsafe {
            core::ptr::copy_nonoverlapping(
                source.as_ptr().add(i * current_frames),
                ch.as_mut_ptr().cast(),
                current_frames,
            );
        }
        // SAFETY: TODO: see above?
        Ok(unsafe { core::slice::from_raw_parts_mut(ch.as_mut_ptr().cast(), ch.len()) })
    })
}

/// Copies contiguous channels into a single slice, interleaving them.
///
/// # Errors
///
/// [`Error::Jagged`] if not all source channels have the same length.
/// [`Error::LengthMismatch`] if the channels don't fit snugly into the destination.
// TODO: test with frames = 0
pub fn copy_to_interleaved<T>(
    source: impl IntoIterator<IntoIter: ExactSizeIterator, Item: Channel<T>>,
    destination: &mut [T],
) -> Result<(), Error>
where
    T: Copy,
{
    // SAFETY: Transmuting &mut [T] to &mut [MaybeUninit<T>] is generally unsafe!
    // However, T implements Copy and only valid T values will ever be written,
    // and the reference never leaves our control, so it should be fine.
    let destination = unsafe { &mut *(destination as *mut [_] as *mut _) };
    copy_to_interleaved_uninit(source, destination).map(|_| {})
}

/// Copies contiguous channels into a single uninitialized slice, interleaving them.
///
/// Same as [`copy_to_interleaved()`], but writing into an uninitialized slice.
///
/// Returns an initialized version of the destination slice on success.
pub fn copy_to_interleaved_uninit<T>(
    source: impl IntoIterator<IntoIter: ExactSizeIterator, Item: Channel<T>>,
    destination: &mut [MaybeUninit<T>],
) -> Result<&mut [T], Error>
where
    T: Copy,
{
    let source = source.into_iter();
    // TODO: move this comment to the docstring?
    // NB: len() is provided by ExactSizeIterator.
    // We could probably implement this without it, but it's simpler
    // and we can show off how to get the number of channels from an iterator.
    let channels = source.len();
    let mut frames = None;
    for (i, ch) in source.enumerate() {
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
        for (dst, src) in destination.iter_mut().skip(i).step_by(channels).zip(ch) {
            *dst = MaybeUninit::new(*src);
        }
    }
    // TODO: return frames & channels?
    // SAFETY: All slice elements have been initialized.
    Ok(unsafe {
        core::slice::from_raw_parts_mut(destination.as_mut_ptr().cast(), destination.len())
    })
}

/// Copies contiguous channels into a single slice, one after another.
///
/// # Errors
///
/// [`Error::Jagged`] if not all source channels have the same length.
/// [`Error::LengthMismatch`] if the channels don't fit snugly into the destination.
// TODO: Regarding ExactSizeIterator, see copy_to_interleaved()
pub fn copy_to_noninterleaved<T>(
    source: impl IntoIterator<IntoIter: ExactSizeIterator, Item: Channel<T>>,
    destination: &mut [T],
) -> Result<(), Error>
where
    T: Copy,
{
    // SAFETY: Transmuting &mut [T] to &mut [MaybeUninit<T>] is generally unsafe!
    // However, T implements Copy and only valid T values will ever be written,
    // and the reference never leaves our control, so it should be fine.
    let destination = unsafe { &mut *(destination as *mut [_] as *mut _) };
    copy_to_noninterleaved_uninit(source, destination).map(|_| ())
}

/// Copies contiguous channels into a single uninitialized slice, one after another.
///
/// Same as [`copy_to_noninterleaved()`], but writing into an uninitialized slice.
///
/// Returns an initialized version of the destination slice on success.
pub fn copy_to_noninterleaved_uninit<T>(
    source: impl IntoIterator<IntoIter: ExactSizeIterator, Item: Channel<T>>,
    destination: &mut [MaybeUninit<T>],
) -> Result<&mut [T], Error>
where
    T: Copy,
{
    let source = source.into_iter();
    let channels = source.len();
    let mut frames = None;
    for (i, ch) in source.enumerate() {
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
        // SAFETY: Source and destination point to the right amount of elements
        // and non-overlapping uninitialized space, respectively.
        unsafe {
            core::ptr::copy_nonoverlapping(
                ch.as_ptr(),
                destination.as_mut_ptr().add(i * current_frames).cast(),
                current_frames,
            );
        }
    }
    // TODO: return frames & channels?
    // SAFETY: All slice elements have been initialized.
    Ok(unsafe {
        core::slice::from_raw_parts_mut(destination.as_mut_ptr().cast(), destination.len())
    })
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

    extern crate alloc;
    use alloc::vec;

    #[test]
    fn test_copy_to_interleaved() {
        let source: [&[_]; _] = [&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]];
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

        // Incorrect usage:
        p.process(&mut [ch0, ch1]);
        assert_eq!(ch0, [1.0, 2.0, 3.0]);

        p.process([&mut ch0, &mut ch1]);
        assert_eq!(ch0, [99.9, 2.0, 3.0]);

        let mut signal = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
        p.process(&mut signal);
        assert_eq!(signal, [[99.9, 2.0, 3.0], [4.0, 5.0, 6.0]]);

        {
            let mut ch0 = vec![1.0, 2.0, 3.0];
            let mut ch1 = vec![4.0, 5.0, 6.0];
            p.process([&mut ch0, &mut ch1]);
            assert_eq!(ch0, [99.9, 2.0, 3.0]);
        }
    }

    #[test]
    fn process_vec() {
        let mut signal = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];
        let mut p = Processor::new();
        p.process(&mut signal);
        assert_eq!(signal, [[99.9, 2.0, 3.0], [4.0, 5.0, 6.0]]);
    }

    #[test]
    fn process_noninterleaved() {
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
        let mut mono = vec![1.0, 2.0, 3.0, 4.0];
        p.process([&mut mono]);
        assert_eq!(mono, [99.9, 2.0, 3.0, 4.0]);
    }
}
