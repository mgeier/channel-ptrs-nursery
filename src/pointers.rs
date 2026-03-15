//! Pointers to channels.
//!
//! These are needed in some C APIs, do not use this in pure Rust code!

use crate::{Channel, ChannelMut, Error};

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
