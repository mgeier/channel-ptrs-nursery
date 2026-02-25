//! TODO.
//!
//! TODO: move this to separate module?
//!
//! ... contiguous channels are common ...
//!
//! # Slice of slices
//!
//! slice of slice is an obvious choice ...
//!
//! TODO: nested slices need extra storage, see e.g. [`pointers::channel_ptrs_to_nested_slices()`]
//!
//! ```
//! pub fn process(channels: &mut [&mut [f32]]) {
//!     for channel in channels {
//!         channel[0] *= 5.0;
//!     }
//! }
//!
//! let mut left = [0.1, 0.2, 0.3];
//! let mut right = [-0.1, -0.2, -0.3];
//! process(&mut [&mut left, &mut right]);
//! assert_eq!(left, [0.5, 0.2, 0.3]);
//! assert_eq!(right, [-0.5, -0.2, -0.3]);
//! ```
//!
//! array/Vec does not work ...
//!
//! # Slice of slice-like structures
//!
//! ```
//! pub fn process(channels: &mut [impl AsMut<[f32]>]) {
//!     for channel in channels {
//!         // Specifying the type is not necessary, but we can be explicit if we want:
//!         let channel: &mut [f32] = channel.as_mut();
//!         channel[0] *= 5.0;
//!     }
//! }
//!
//! let mut data = [[0.1, 0.2, 0.3], [-0.1, -0.2, -0.3]];
//! process(&mut data);
//! assert_eq!(data, [[0.5, 0.2, 0.3], [-0.5, -0.2, -0.3]]);
//!
//! let mut vec_data = vec![vec![0.1, 0.2, 0.3], vec![-0.1, -0.2, -0.3]];
//! process(&mut vec_data);
//! assert_eq!(vec_data, [[0.5, 0.2, 0.3], [-0.5, -0.2, -0.3]]);
//! ```
//!
//! This is a fine solution (with a reasonably simple implementation),
//! but if we are willing to put in some more work, we can make our `process()` function
//! even more flexible.
//!
//! # Iterator over slice-like structures
//!
//! But things will get worse before they get better again ...
//!
//! ```
//! pub fn process(channels: impl IntoIterator<Item: AsMut<[f32]>>) {
//!     for mut channel in channels {
//!         let channel = channel.as_mut();
//!         channel[0] *= 5.0;
//!     }
//! }
//!
//! let mut noninterleaved = [0.1, 0.2, 0.3, -0.1, -0.2, -0.3];
//! process(noninterleaved.chunks_mut(3));
//! assert_eq!(noninterleaved, [0.5, 0.2, 0.3, -0.5, -0.2, -0.3]);
//!
//! // however, this is problematic ...
//!
//! let mut data = [[0.1, 0.2, 0.3], [-0.1, -0.2, -0.3]];
//! process(data);
//! assert_eq!(data, [[0.1, 0.2, 0.3], [-0.1, -0.2, -0.3]]); // NO CHANGE!
//!
//! // note that the data is unchanged ... since array is `Copy` ...
//! ```
//!
//! ... we can avoid this by adding more stuff ...
//!
//! ```
//! pub fn process<'c, C>(channels: impl IntoIterator<Item = &'c mut C>)
//! where
//!     C: AsMut<[f32]> + ?Sized + 'c,
//! {
//!     for mut channel in channels {
//!         let channel = channel.as_mut();
//!         channel[0] *= 5.0;
//!     }
//! }
//!
//! let mut data = [[0.1, 0.2, 0.3], [-0.1, -0.2, -0.3]];
//! process(&mut data);
//! assert_eq!(data, [[0.5, 0.2, 0.3], [-0.5, -0.2, -0.3]]);
//!
//! // The `?Sized` part above is needed for this to work:
//! let mut noninterleaved = [0.1, 0.2, 0.3, -0.1, -0.2, -0.3];
//! process(noninterleaved.chunks_mut(3));
//! assert_eq!(noninterleaved, [0.5, 0.2, 0.3, -0.5, -0.2, -0.3]);
//! ```
//!
//! ... this does what we want, but the function signature is getting quite cryptic ...
//!
//! # Iterator over channels
//!
//! ... let's try to make the function signature less cryptic by introducing
//! a trivial-looking trait and a cryptic blanket implementation ...
//!
//! ```
//! trait ChannelMut<T>: AsMut<[T]> {}
//! impl<T, U: AsMut<[T]> + ?Sized> ChannelMut<T> for &mut U {}
//!
//! // With this, the function signature arguably looks less intimidating:
//!
//! pub fn process(channels: impl IntoIterator<Item: ChannelMut<f32>>) {
//!     for mut channel in channels {
//!         let channel = channel.as_mut();
//!         channel[0] *= 5.0;
//!     }
//! }
//!
//! // This allows all the usage scenarios we've seen before.
//!
//! let mut data = [[0.1, 0.2, 0.3], [-0.1, -0.2, -0.3]];
//! process(&mut data);
//! assert_eq!(data, [[0.5, 0.2, 0.3], [-0.5, -0.2, -0.3]]);
//!
//! let mut vec_data = vec![vec![0.1, 0.2, 0.3], vec![-0.1, -0.2, -0.3]];
//! process(&mut vec_data);
//! assert_eq!(vec_data, [[0.5, 0.2, 0.3], [-0.5, -0.2, -0.3]]);
//!
//! let mut left = [0.1, 0.2, 0.3];
//! let mut right = [-0.1, -0.2, -0.3];
//! process(&mut [&mut left, &mut right]);
//! assert_eq!(left, [0.5, 0.2, 0.3]);
//! assert_eq!(right, [-0.5, -0.2, -0.3]);
//!
//! let mut noninterleaved = [0.1, 0.2, 0.3, -0.1, -0.2, -0.3];
//! process(noninterleaved.chunks_mut(3));
//! assert_eq!(noninterleaved, [0.5, 0.2, 0.3, -0.5, -0.2, -0.3]);
//! ```
//!
//! This doesn't look too bad, does it?
//!
//! For your convenience, we're providing this trait (and the corresponding blanket implementation)
//! here: [`ChannelMut`] (as well as its non-mutable counterpart [`Channel`]).
//!
//! # Just channels
//!
//! If you want to make the function signature more concise (but also less explicit!),
//! you can do this at the cost of defining yet another trait with a blanket implementation:
//!
//! ```
//! trait ChannelMut<T>: AsMut<[T]> {}
//! impl<T, U: AsMut<[T]> + ?Sized> ChannelMut<T> for &mut U {}
//!
//! trait ChannelsMut<T>: IntoIterator<Item: ChannelMut<T>> {}
//! impl<T, U: IntoIterator<Item: ChannelMut<T>>> ChannelsMut<T> for U {}
//!
//! pub fn process(channels: impl ChannelsMut<f32>) {
//!     for mut channel in channels {
//!         let channel = channel.as_mut();
//!         channel[0] *= 5.0;
//!     }
//! }
//!
//! // This allows the same usage scenarios as before, we're trying just a few here.
//!
//! let mut data = [[0.1, 0.2, 0.3], [-0.1, -0.2, -0.3]];
//! process(&mut data);
//! assert_eq!(data, [[0.5, 0.2, 0.3], [-0.5, -0.2, -0.3]]);
//!
//! let mut noninterleaved = [0.1, 0.2, 0.3, -0.1, -0.2, -0.3];
//! process(noninterleaved.chunks_mut(3));
//! assert_eq!(noninterleaved, [0.5, 0.2, 0.3, -0.5, -0.2, -0.3]);
//! ```
//!
//! # Different flavors of iterators
//!
//! TODO: ExactSizeIterator, see e.g. [`flat::copy_to_interleaved()`] ...
//!
//! TODO: Clone, DoubleEndedIterator, other traits ... users can create their own custom
//! `Channels` and `ChannelsMut` traits (together with appropriate blanket implementations).
//! We're intentionally not providing those traits here.

#![no_std]
#![forbid(clippy::undocumented_unsafe_blocks)]

pub mod flat;
pub mod frames;
#[cfg(feature = "ndarray")]
pub mod ndarray;
pub mod pointers;

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
