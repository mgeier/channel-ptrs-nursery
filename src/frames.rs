//! Iterating over frames.

use core::marker::PhantomData;

/// Returns an iterator over frames (which themselves are iterators over samples).
// TODO: docs: this does not copy any samples and instead yields `&T`
// TODO: docs: depending on the circumstances, it might be more efficient to copy all samples to interleaved first and then use `chunks()` to iterate over frames
// see [`copy_to_interleaved()`]
// TODO: docs: [`Channel`] cannot be used because it cannot express necessary lifetimes?
#[must_use]
pub fn frames_from_channels<'a, T, I, C>(channels: I) -> FramesFromChannels<'a, T, I, C>
where
    I: IntoIterator<Item = &'a C> + Clone,
    C: AsRef<[T]> + ?Sized,
{
    FramesFromChannels {
        index: 0,
        frames: None,
        channels,
        _phantom: PhantomData,
    }
}

pub struct FramesFromChannels<'a, T, I, C>
where
    I: IntoIterator<Item = &'a C> + Clone,
    C: AsRef<[T]> + ?Sized + 'a,
{
    index: usize,
    frames: Option<usize>,
    channels: I,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T, I, C> Iterator for FramesFromChannels<'a, T, I, C>
where
    I: IntoIterator<Item = &'a C> + Clone,
    C: AsRef<[T]> + ?Sized + 'a,
{
    type Item = FrameFromChannels<'a, T, I::IntoIter, C>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frames.is_none() {
            self.frames = Some(self.channels.clone().into_iter().next()?.as_ref().len());
        }
        if self.index < self.frames.unwrap() {
            let index = self.index;
            self.index += 1;
            Some(FrameFromChannels {
                index,
                channels: self.channels.clone().into_iter(),
                _phantom: PhantomData,
            })
        } else {
            None
        }
    }
}

pub struct FrameFromChannels<'a, T, I, C>
where
    I: Iterator<Item = &'a C>,
    C: AsRef<[T]> + ?Sized + 'a,
{
    index: usize,
    channels: I,
    _phantom: PhantomData<&'a (T, C)>,
}

impl<'a, T, I, C> Iterator for FrameFromChannels<'a, T, I, C>
where
    I: Iterator<Item = &'a C>,
    C: AsRef<[T]> + ?Sized + 'a,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.channels
            .next()
            .map(|ch| ch.as_ref().get(self.index).unwrap())
    }
}

/// Returns a pseudo-iterator over writable frames (which themselves are iterators over samples).
///
/// TODO: not compatible with [`Iterator`],
/// `LendingIterator` doesn't exist yet in the standard library
///
/// `channels` is *not* an [`IntoIterator`], because the channels are iterated multiple times
/// (once for each frame), which would require [`Copy`], which is not available for mutable
/// slices/containers.
#[must_use]
pub fn frames_from_channels_mut<'a, T, C>(channels: &'a mut [C]) -> FramesFromChannelsMut<'a, T, C>
where
    C: AsMut<[T]>,
{
    FramesFromChannelsMut {
        index: 0,
        frames: None,
        channels,
        _phantom: PhantomData,
    }
}

pub struct FramesFromChannelsMut<'a, T, C> {
    index: usize,
    frames: Option<usize>,
    channels: &'a mut [C],
    _phantom: PhantomData<&'a mut T>,
}

impl<T, C> FramesFromChannelsMut<'_, T, C>
where
    C: AsMut<[T]>,
{
    pub fn next_frame(&mut self) -> Option<impl Iterator<Item = &mut T>> {
        if self.frames.is_none() {
            self.frames = Some(self.channels.first_mut()?.as_mut().len());
        }
        if self.index < self.frames.unwrap() {
            let index = self.index;
            self.index += 1;
            Some(
                self.channels
                    .iter_mut()
                    .map(move |ch| ch.as_mut().get_mut(index).unwrap()),
            )
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    extern crate alloc;
    use alloc::vec;

    #[test]
    fn test_vec_frames() {
        let v = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];
        let frames = frames_from_channels(&v);
        for frame in frames {
            for (_i, _s) in frame.enumerate() {
                // TODO: test something! collect?
            }
        }
    }

    #[test]
    fn test_noninterleaved_frames() {
        let a = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let frames = frames_from_channels(a.chunks(3));
        for frame in frames {
            for (_i, _s) in frame.enumerate() {
                // TODO: test something! collect?
            }
        }
    }

    // NB: this is not a common use case, but it helps us make sure that no copies are made.
    #[test]
    fn test_vec_cell_frames() {
        use core::cell::Cell;

        let v = vec![
            vec![Cell::new(1.0), Cell::new(2.0), Cell::new(3.0)],
            vec![Cell::new(4.0), Cell::new(5.0), Cell::new(6.0)],
        ];
        let frames = frames_from_channels(&v);
        for frame in frames {
            for (i, s) in frame.enumerate() {
                s.update(|x| x * 100.0 + i as f32);
            }
        }
        assert_eq!(
            v,
            [
                [Cell::new(100.0), Cell::new(200.0), Cell::new(300.0)],
                [Cell::new(401.0), Cell::new(501.0), Cell::new(601.0)]
            ]
        );
    }

    #[test]
    fn test_vec_frames_mut() {
        let mut v = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];
        let mut frames = frames_from_channels_mut(&mut v);
        while let Some(frame) = frames.next_frame() {
            for (i, s) in frame.enumerate() {
                *s = *s * 100.0 + i as f32;
            }
        }
        assert_eq!(v, [[100.0, 200.0, 300.0], [401.0, 501.0, 601.0]]);
    }
}
