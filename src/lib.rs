//! Handling Multi-Channel Audio Signals.
//!
//! ... this is about interfaces between components ...
//! within the private implementation of a component, other approaches might be
//! useful (e.g. operator overloading for block-wise calculations) ...
//!
//! ... block-based vs. sample/frame-based ...
//!
//! ... contiguous channels are common ...
//!
//! ... alternative: (non)interleaved channels in flat slices, see [`flat`] module
//!
//! ... when interfacing with Python, often two-dimensional NumPy arrays are used.
//! those are basically interleaved or noninterleaved ...
//! See [`ndarray`] module.
//!
//! ... when interfacing via FFI, often pointers to pointers are used,
//! see [`pointers`] module
//!
//!
//! # Slice of slices
//!
//! A block of a multi-channel audio signal can be seen as a sequence of channels.
//! It often makes sense to store one channel of such a block
//! in contiguous memory, which can be easily passed around by means of a
//! (mutable or immutable) [`slice`].
//! The whole multi-channel block can then be represented by a
//! slice of slices:
//!
//! ```
//! fn process(channels: &mut [&mut [f32]]) {
//!     // Instead of iterating, we could also use indexing.
//!     for channel in channels {
//!         // Instead of indexing, we could also use iteration.
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
//! Of course we don't have to use `f32`, we can use any other available sample type
//! or a generic type, if desired.
//!
//! In this example we are passing a mutable slice of slices (`&mut [&mut [f32]]`),
//! but if only read access is needed, we can also use an immutable slice of slices
//! (`&[&[f32]]`).
//!
//!
//! # Slice of slice-like structures
//!
//! A big disadvantage of the previous approach is that it doesn't work with signals
//! stored in nested [`Vec`]s or [`array`]s.
//! To allow this, we can extend it by using [`AsMut`]
//! (or [`AsRef`] for immutable arguments) and turn it into a slice by calling
//! the `.as_mut()` method (or `.as_ref()`, respectively).
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
//! // The previous example still works, but now also this works:
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
//! but it still has (at least) two downsides:
//!
//! * To create the outer slice, often extra storage (and maybe even dynamic allocation) is needed,
//!   see e.g. [`pointers::channel_ptrs_to_nested_slices()`].
//! * It doesn't support iterators over channels, e.g. like the one created by
//!   [`slice::chunks_mut()`].
//!
//!
//! # Iterator over slice-like structures
//!
//! If we are willing to put in some more work, we can make our `process()` function
//! even more flexible.
//! But things will get worse before they get better again ...
//!
//! ```
//! pub fn process(channels: impl IntoIterator<Item: AsMut<[f32]>>) {
//!     // Use in for-loop or call .into_iter():
//!     for mut channel in channels {
//!         // Call .as_mut() on each channel to get a "normal" writable slice:
//!         let channel = channel.as_mut();
//!         channel[0] *= 5.0;
//!     }
//! }
//!
//! // Now it works with iterators:
//!
//! let mut noninterleaved = [0.1, 0.2, 0.3, -0.1, -0.2, -0.3];
//! process(noninterleaved.chunks_mut(3));
//! assert_eq!(noninterleaved, [0.5, 0.2, 0.3, -0.5, -0.2, -0.3]);
//!
//! // And this becomes a bit simpler (one fewer `&mut`):
//!
//! let mut left = [0.1, 0.2, 0.3];
//! let mut right = [-0.1, -0.2, -0.3];
//! process([&mut left, &mut right]);
//! assert_eq!(left, [0.5, 0.2, 0.3]);
//! assert_eq!(right, [-0.5, -0.2, -0.3]);
//!
//! // However, this is problematic:
//!
//! let mut data = [[0.1, 0.2, 0.3], [-0.1, -0.2, -0.3]];
//! process(data);
//! assert_eq!(data, [[0.1, 0.2, 0.3], [-0.1, -0.2, -0.3]]); // NO CHANGE!
//! ```
//!
//! Note that `data` is unchanged, even though `process()` is supposed to change it!
//! This happens because [`array` implements `Copy`](array#impl-Copy-for-%5BT;+N%5D),
//! which means that the whole audio block is copied before it is passed to `process()`!
//!
//! We can avoid this by adding more stuff:
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
//! // Now we are forced to pass a `&mut` (which is good!):
//!
//! let mut data = [[0.1, 0.2, 0.3], [-0.1, -0.2, -0.3]];
//! //process(data); // compiler error: expected mutable reference `&mut _`
//! process(&mut data);
//! assert_eq!(data, [[0.5, 0.2, 0.3], [-0.5, -0.2, -0.3]]);
//!
//! // Iterators still work, but they need the `?Sized` part above:
//!
//! let mut noninterleaved = [0.1, 0.2, 0.3, -0.1, -0.2, -0.3];
//! process(noninterleaved.chunks_mut(3));
//! assert_eq!(noninterleaved, [0.5, 0.2, 0.3, -0.5, -0.2, -0.3]);
//! ```
//!
//! This works for all the situations we have encountered so far,
//! but the function signature is getting quite cryptic, isn't it?
//!
//! The following section shows how to avoid the intimidating explicit lifetime annotations
//! and `where` clauses, but in some special cases we will not be able to avoid them, see e.g.
//! [`frames::frames_from_channels()`] and [`frames::frames_from_channels_mut()`].
//!
//!
//! # Iterator over channels
//!
//! Let's try to make the function signature less cryptic
//! by introducing a trivial-looking trait and a cryptic
//! [blanket implementation](https://doc.rust-lang.org/book/ch10-02-traits.html#using-trait-bounds-to-conditionally-implement-methods):
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
//! process([&mut left, &mut right]);
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
//! // This allows the same usage scenarios as before, we're checking just a few here.
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
//!
//! # Downsides of iterators
//!
//! Iterators are very flexible, but ...
//!
//! TODO: iterate only once, except when [`Clone`] (which isn't possible for writable slices)
//!
//! TODO: example: [`frames::frames_from_channels()`] (`Clone`)
//!
//! TODO: example: [`frames::frames_from_channels_mut()`] (slice of slices instead of iterator of
//! slices)

#![forbid(clippy::undocumented_unsafe_blocks)]

pub mod flat;
pub mod frames;
#[cfg(feature = "ndarray")]
pub mod ndarray;
pub mod pointers;

// TODO: iter to slice?

/// A non-mutable audio channel in contiguous memory.
///
/// Nobody needs to implement this trait because it already has a
/// [blanket implementation](Channel#impl-Channel%3CT%3E-for-%26U).
///
/// This can be used to define generic function arguments
/// (using `impl IntoIterator<Item: Channel<T>>`) that accept multi-channel signals.
///
/// TODO: mutable: [`ChannelMut`]
///
/// TODO: mention ExactSizeIterator?
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
///
/// For example usage, see e.g. [`flat::copy_to_interleaved()`] (featuring [`ExactSizeIterator`])
/// and [`pointers::channel_ptrs_from_slices()`].
pub trait Channel<T>: AsRef<[T]> {}

/// This [blanket implementation](https://doc.rust-lang.org/book/ch10-02-traits.html#using-trait-bounds-to-conditionally-implement-methods)
/// automatically implements the `Channel` trait for all (immutable) slice-like types.
impl<T, U: AsRef<[T]> + ?Sized> Channel<T> for &U {}

/// A mutable audio channel in contiguous memory.
///
/// Nobody needs to implement this trait because it already has a
/// [blanket implementation](ChannelMut#impl-ChannelMut%3CT%3E-for-%26mut+U).
///
/// This can be used to define generic function arguments
/// (using `impl IntoIterator<Item: ChannelMut<T>>`) that accept multi-channel signals.
///
/// TODO: immutable: [`Channel`]
///
/// # Examples
///
/// For example usage, see e.g. [`flat::copy_from_interleaved()`] (featuring [`ExactSizeIterator`])
/// and [`pointers::channel_ptrs_from_slices_mut()`].
///
/// See [crate-level documentation](crate#iterator-over-channels) for more examples.
pub trait ChannelMut<T>: AsMut<[T]> {}

/// This [blanket implementation](https://doc.rust-lang.org/book/ch10-02-traits.html#using-trait-bounds-to-conditionally-implement-methods)
/// automatically implements the `ChannelMut` trait for all (mutable) slice-like types.
impl<T, U: AsMut<[T]> + ?Sized> ChannelMut<T> for &mut U {}

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    // all channels must have the same length
    Jagged,
    LengthMismatch,
    // too few pointers in `storage`
    StorageOverflow,
}
