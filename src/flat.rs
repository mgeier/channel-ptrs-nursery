//! Storing multiple channels in a single "flat" slice.
//!
//! There are two common ways to do this: *interleaved* and *non-interleaved*.
//! When storing the channels *interleaved*, we first store the first sample of each channel,
//! then the second sample of each channel, and so on.
//! When storing them *non-interleaved*, we first store all samples of the first channel,
//! then all samples of the second channel, and so on.
//!
//! To illustrate this, let's say we have an audio block with 3 channels (`A`, `B` and `C`)
//! and a block length of 5 (i.e. each channel has 5 samples).
//! This is what it would look like with *interleaved* storage:
//!
//! ```text
//! [A0, B0, C0, A1, B1, C1, A2, B2, C2, A3, B3, C3, A4, B4, C4]
//! ```
//!
//! This is what it would look like with *non-interleaved* storage:
//!
//! ```text
//! [A0, A1, A2, A3, A4, B0, B1, B2, B3, B4, C0, C1, C2, C3, C4]
//! ```
//!
//! TODO: contiguous frames vs. contiguous channels, chunks() and chunks_mut(),
//! [`much::frames`](crate::frames)
//!
//! TODO: link to [`much::ndarray`](crate::ndarray), relation to column vs. row major

use core::mem::MaybeUninit;

use crate::{Channel, ChannelMut, Error};

/// Copies all samples from a single slice of interleaved channels into contiguous channels.
///
/// If you want to copy into uninitialized channels, you can use
/// [`copy_from_interleaved_uninit()`].
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
///
/// When this returns successfully, all channels in `destination` have been fully initialized.
/// If you need access to those initialized channels afterwards, you can use
/// [`copy_from_interleaved_uninit_and_iterate()`] instead.
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
/// If you want to copy into uninitialized channels, you can use
/// [`copy_from_noninterleaved_uninit()`].
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
///
/// When this returns successfully, all channels in `destination` have been fully initialized.
/// If you need access to those initialized channels afterwards, you can use
/// [`copy_from_noninterleaved_uninit_and_iterate()`] instead.
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
/// If you want to copy into an uninitialized slice, you can use
/// [`copy_to_interleaved_uninit()`].
///
/// If you want to copy without interleaving, you can use
/// [`copy_to_noninterleaved()`].
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
/// If you want to copy into an uninitialized slice, you can use
/// [`copy_to_noninterleaved_uninit()`].
///
/// If you want to interleave the channels, you can use
/// [`copy_to_interleaved()`].
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_to_interleaved() {
        let source: [&[_]; _] = [&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]];
        let mut destination = [0.0; 6];
        copy_to_interleaved(source, &mut destination).unwrap();
        assert_eq!(destination, [1.0, 4.0, 2.0, 5.0, 3.0, 6.0]);
    }
}
