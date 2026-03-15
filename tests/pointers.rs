use much::ChannelMut;
use much::pointers::channel_ptrs_from_slices_mut;

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
    pub fn process(&mut self, signal: impl IntoIterator<Item: ChannelMut<f32>>) {
        let (ptrs, frames, channels) =
            channel_ptrs_from_slices_mut(signal, &mut self.channel_ptrs).unwrap();

        // SAFETY: channel_ptrs_from_slices_mut() returned valid results.
        unsafe {
            set_a_value(ptrs, frames, channels);
        }
    }
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
