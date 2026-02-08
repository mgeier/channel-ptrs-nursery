#![cfg(feature = "ndarray")]

use much::ndarray::{
    contiguous_columns_mut, interleaved_columns_mut,
};
use ndarray::ArrayRef2;

fn _process_columns_inplace(a: &mut ArrayRef2<f32>) {
    if let Some(_iter) = contiguous_columns_mut(a) {
        todo!()
    } else if let Some(_slice) = interleaved_columns_mut(a) {
        todo!()
    } else {
        // TODO: error (read-only array would be copied to appropriate layout)
    }

    // TODO: explain behavior when one-channel signal is given, interleaved or not?
}

fn main() {
    /*
    let mut column_vector = array![[0.1f32], [0.2], [0.3]];

    {
        let mut iter = contiguous_rows(&column_vector).unwrap();
        assert_eq!(iter.next().unwrap(), [0.1]);
        assert_eq!(iter.next().unwrap(), [0.2]);
        assert_eq!(iter.next().unwrap(), [0.3]);
        assert!(iter.next().is_none());
    }

    let s = interleaved_columns_mut(&mut column_vector).unwrap();
    s[1] = -0.2;
    let s = interleaved_columns(&column_vector).unwrap();
    assert_eq!(s, [0.1, -0.2, 0.3]);

    let s = interleaved_rows(&column_vector).unwrap();
    assert_eq!(s, [0.1, -0.2, 0.3]);

    let mut row_vector = array![[0.1f32, 0.2, 0.3]];

    let s = interleaved_columns_mut(&mut row_vector).unwrap();
    s[2] = -0.3;
*/
}
