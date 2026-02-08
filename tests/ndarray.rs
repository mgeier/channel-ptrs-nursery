#![cfg(feature = "ndarray")]

use much::ndarray::{
    contiguous_columns, contiguous_columns_mut, contiguous_rows, contiguous_rows_mut,
    interleaved_columns, interleaved_columns_mut, interleaved_rows, interleaved_rows_mut,
};
use ndarray::{Array, ShapeBuilder, array};

// Channels contain increasing numbers. Channel 0 is positive, channel 1 is negative etc.
fn column_data((r, c): (usize, usize)) -> f32 {
    r as f32 * if c % 2 == 0 { 0.1 } else { -0.1 }
}

// Channels contain increasing numbers. Channel 0 is positive, channel 1 is negative etc.
fn row_data((r, c): (usize, usize)) -> f32 {
    c as f32 * if r % 2 == 0 { 0.1 } else { -0.1 }
}

// Channels as columns, C memory order.
#[test]
fn columns_c() {
    let mut a = Array::from_shape_fn((3, 2), column_data);
    assert_eq!(a, array![[0.0, -0.0], [0.1, -0.1], [0.2, -0.2]]);
    assert!(a.is_standard_layout());
    assert!(contiguous_columns(&a).is_none());
    assert!(contiguous_columns_mut(&mut a).is_none());
    assert!(interleaved_rows(&a).is_none());
    assert!(interleaved_rows_mut(&mut a).is_none());

    // Iterate over frames.
    {
        let mut iter = contiguous_rows_mut(&mut a).unwrap();
        let row0 = iter.next().unwrap();
        assert_eq!(row0, [0.0, -0.0]);
        let row1 = iter.next().unwrap();
        row1[0] = 99.9;
        let row2 = iter.next().unwrap();
        assert_eq!(row2, [0.2, -0.2]);
    }
    {
        let mut iter = contiguous_rows(&a).unwrap();
        let row0 = iter.next().unwrap();
        assert_eq!(row0, [0.0, -0.0]);
        let row1 = iter.next().unwrap();
        assert_eq!(row1, [99.9, -0.1]);
        let row2 = iter.next().unwrap();
        assert_eq!(row2, [0.2, -0.2]);
    }

    // Access interleaved channels.
    let data = interleaved_columns_mut(&mut a).unwrap();
    data[3] = -99.9;
    let data = interleaved_columns(&a).unwrap();
    assert_eq!(data, [0.0, -0.0, 99.9, -99.9, 0.2, -0.2]);
}

// Channels as columns, Fortran memory order.
#[test]
fn columns_f() {
    let mut a = Array::from_shape_fn((3, 2).f(), column_data);
    assert_eq!(a, array![[0.0, -0.0], [0.1, -0.1], [0.2, -0.2]]);
    assert!(!a.is_standard_layout());
    assert!(contiguous_rows(&a).is_none());
    assert!(contiguous_rows_mut(&mut a).is_none());
    assert!(interleaved_columns(&a).is_none());
    assert!(interleaved_columns_mut(&mut a).is_none());

    // Iterate over channels.
    {
        let mut iter = contiguous_columns_mut(&mut a).unwrap();
        let column0 = iter.next().unwrap();
        assert_eq!(column0, [0.0, 0.1, 0.2]);
        let column1 = iter.next().unwrap();
        column1[1] = -99.9;
    }
    {
        let mut iter = contiguous_columns(&a).unwrap();
        let column0 = iter.next().unwrap();
        assert_eq!(column0, [0.0, 0.1, 0.2]);
        let column1 = iter.next().unwrap();
        assert_eq!(column1, [-0.0, -99.9, -0.2]);
    }

    // Access interleaved frames (a.k.a. non-interleaved channels).
    let data = interleaved_rows_mut(&mut a).unwrap();
    data[1] = 99.9;
    let data = interleaved_rows(&a).unwrap();
    assert_eq!(data, [0.0, 99.9, 0.2, -0.0, -99.9, -0.2]);
}

// Channels as rows, C memory order.
#[test]
fn rows_c() {
    let mut a = Array::from_shape_fn((2, 3), row_data);
    assert_eq!(a, array![[0.0, 0.1, 0.2], [-0.0, -0.1, -0.2]]);
    assert!(a.is_standard_layout());
    assert!(contiguous_columns(&a).is_none());
    assert!(contiguous_columns_mut(&mut a).is_none());
    assert!(interleaved_rows(&a).is_none());
    assert!(interleaved_rows_mut(&mut a).is_none());

    // Iterate over channels.
    {
        let mut iter = contiguous_rows_mut(&mut a).unwrap();
        let row0 = iter.next().unwrap();
        assert_eq!(row0, [0.0, 0.1, 0.2]);
        let row1 = iter.next().unwrap();
        row1[1] = -99.9;
    }
    {
        let mut iter = contiguous_rows(&a).unwrap();
        let row0 = iter.next().unwrap();
        assert_eq!(row0, [0.0, 0.1, 0.2]);
        let row1 = iter.next().unwrap();
        assert_eq!(row1, [-0.0, -99.9, -0.2]);
    }

    // Access interleaved frames (a.k.a. non-interleaved channels).
    let data = interleaved_columns_mut(&mut a).unwrap();
    data[1] = 99.9;
    let data = interleaved_columns(&a).unwrap();
    assert_eq!(data, [0.0, 99.9, 0.2, -0.0, -99.9, -0.2]);
}

// Channels as rows, Fortran memory order.
#[test]
fn rows_f() {
    let mut a = Array::from_shape_fn((2, 3).f(), row_data);
    assert_eq!(a, array![[0.0, 0.1, 0.2], [-0.0, -0.1, -0.2]]);
    assert!(!a.is_standard_layout());
    assert!(contiguous_rows(&a).is_none());
    assert!(contiguous_rows_mut(&mut a).is_none());
    assert!(interleaved_columns(&a).is_none());
    assert!(interleaved_columns_mut(&mut a).is_none());

    // Iterate over frames.
    {
        let mut iter = contiguous_columns_mut(&mut a).unwrap();
        let column0 = iter.next().unwrap();
        assert_eq!(column0, [0.0, -0.0]);
        let column1 = iter.next().unwrap();
        column1[0] = 99.9;
        let column2 = iter.next().unwrap();
        assert_eq!(column2, [0.2, -0.2]);
    }
    {
        let mut iter = contiguous_columns(&a).unwrap();
        let column0 = iter.next().unwrap();
        assert_eq!(column0, [0.0, -0.0]);
        let column1 = iter.next().unwrap();
        assert_eq!(column1, [99.9, -0.1]);
        let column2 = iter.next().unwrap();
        assert_eq!(column2, [0.2, -0.2]);
    }

    // Access interleaved channels.
    let data = interleaved_rows_mut(&mut a).unwrap();
    data[3] = -99.9;
    let data = interleaved_rows(&a).unwrap();
    assert_eq!(data, [0.0, -0.0, 99.9, -99.9, 0.2, -0.2]);
}

// TODO: column/row vectors (see example)

// TODO: iterate multiple times (i.e. clone() the IntoIterator?)
