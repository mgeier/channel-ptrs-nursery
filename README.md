Handling Multi-Channel Audio Signals in Rust
============================================

> Don't define containers,  
> define interfaces.

—Irish proverb


Nomenclature
------------

TODO: "frame", "sample"

The term "planar" seems to originate from
[FFMPEG](https://ffmpeg.org/doxygen/trunk/group__lavu__sampfmts.html)
(which also seems to use the term "packed").
It is also used in
[Web Audio](https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API/Basic_concepts_behind_Web_Audio_API).


Potential Alternatives
----------------------

In alphabetical order.
Comments might be outdated (see version numbers in links).

* [audio](https://docs.rs/audio/0.2.1/audio/),
  formerly known as [rotary](https://docs.rs/rotary/0.28.1/rotary/)
  * "dynamic", "interleaved", "sequential"
  * traits: `Buf`, `BufMut`, ...
* [audioadapter](https://docs.rs/audioadapter/2.0.0/audioadapter/) and
  [audioadapter-buffers](https://docs.rs/audioadapter-buffers/2.0.0/audioadapter_buffers/)
  * "interleaved", "sequential"
* [audio-blocks](https://docs.rs/audio-blocks/0.6.0/audio_blocks/)
  * "planar", "sequential", "interleaved", "mono"
  * traits: `AudioBlock`, `AudioBlockMut`, `Sample`
* [audio_buffer_interface](https://docs.rs/crate/audio_buffer_interface/0.1.0)
* [audio-channel-buffer](https://docs.rs/audio-channel-buffer/0.2.4/audio_channel_buffer/)
  * not a lot of documentation, how are those buffers supposed to be passed around?
* [audiochannelutils](https://docs.rs/audiochannelutils/0.0.4/audioutils/)
  * free functions FTW!
  * a lot of `Vec`s in API
  * trait `SampleType`
* [audio_samples](https://docs.rs/audio_samples/0.10.7/audio_samples/)
  * `sample_rate: NonZeroU32`
  * `Interleaved`, `NonInterleaved`
  * `ChannelIterator`
  * good documentation, but feature creep, many dependencies
* [dasp](https://docs.rs/dasp/0.11.0/dasp/),
  formerly known as [sample](https://docs.rs/sample/0.11.0/sample/)
  * no block-wise processing, only frame-wise?


Existing APIs
-------------

It might be interesting to look at existing APIs to see which terms they use,
which data types etc.

* [CPAL](https://docs.rs/cpal/0.17.1/cpal/)
  * traits: `Sample`, ...
  * input/output streams with callbacks taking `&[T]` and `&mut [T]`
    * interleaved data (see also [#367](https://github.com/RustAudio/cpal/issues/367))
    * no duplex support yet (see [#349](https://github.com/RustAudio/cpal/issues/349)
      and [#1096](https://github.com/RustAudio/cpal/pull/1096))
  * types:
    ```
    pub type ChannelCount = u16;
    pub type FrameCount = u32;
    pub type SampleRate = u32;
    pub enum BufferSize {
        Default,
        Fixed(FrameCount),
    }
    ```
  * parameter names:
    ```
    channels: ChannelCount
    sample_rate: SampleRate
    buffer_size: BufferSize
    ```
