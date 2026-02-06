use ndarray::{Array, ArrayRef2, Axis, ShapeBuilder, array};

const COLUMN_AXIS: Axis = Axis(0);
const ROW_AXIS: Axis = Axis(1);

pub fn contiguous_columns<T>(
    a: &ArrayRef2<T>,
) -> Option<impl ExactSizeIterator<Item = &[T]> + DoubleEndedIterator> {
    if a.stride_of(COLUMN_AXIS) != 1 {
        return None;
    }
    Some(
        a.columns()
            .into_iter()
            .map(|c| c.to_slice_memory_order().unwrap()),
    )
}

pub fn contiguous_columns_mut<T>(
    a: &mut ArrayRef2<T>,
) -> Option<impl ExactSizeIterator<Item = &mut [T]> + DoubleEndedIterator> {
    if a.stride_of(COLUMN_AXIS) != 1 {
        return None;
    }
    Some(
        a.columns_mut()
            .into_iter()
            .map(|c| c.into_slice_memory_order().unwrap()),
    )
}

pub fn contiguous_rows<T>(
    a: &ArrayRef2<T>,
) -> Option<impl ExactSizeIterator<Item = &[T]> + DoubleEndedIterator> {
    if a.stride_of(ROW_AXIS) != 1 {
        return None;
    }
    Some(
        a.rows()
            .into_iter()
            .map(|c| c.to_slice_memory_order().unwrap()),
    )
}

pub fn contiguous_rows_mut<T>(
    a: &mut ArrayRef2<T>,
) -> Option<impl ExactSizeIterator<Item = &mut [T]> + DoubleEndedIterator> {
    if a.stride_of(ROW_AXIS) != 1 {
        return None;
    }
    Some(
        a.rows_mut()
            .into_iter()
            .map(|c| c.into_slice_memory_order().unwrap()),
    )
}

pub fn interleaved_columns<T>(a: &ArrayRef2<T>) -> Option<&[T]> {
    if a.stride_of(ROW_AXIS) != 1 {
        return None;
    }
    a.as_slice_memory_order()
}

pub fn interleaved_columns_mut<T>(a: &mut ArrayRef2<T>) -> Option<&mut [T]> {
    if a.stride_of(ROW_AXIS) != 1 {
        return None;
    }
    a.as_slice_memory_order_mut()
}

pub fn interleaved_rows<T>(a: &ArrayRef2<T>) -> Option<&[T]> {
    if a.stride_of(COLUMN_AXIS) != 1 {
        return None;
    }
    a.as_slice_memory_order()
}

pub fn interleaved_rows_mut<T>(a: &mut ArrayRef2<T>) -> Option<&mut [T]> {
    if a.stride_of(COLUMN_AXIS) != 1 {
        return None;
    }
    a.as_slice_memory_order_mut()
}

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

fn column_data((r, c): (usize, usize)) -> f32 {
    r as f32 * if c % 2 == 0 { 0.1 } else { -0.1 }
}

fn main() {
    // Channels as columns, C memory order.
    let mut columns_c = Array::from_shape_fn((3, 2), column_data);
    assert_eq!(columns_c, array![[0.0, -0.0], [0.1, -0.1], [0.2, -0.2]]);
    assert!(columns_c.is_standard_layout());

    assert!(contiguous_columns(&columns_c).is_none());
    assert!(contiguous_columns_mut(&mut columns_c).is_none());

    // Iterate over frames.
    {
        let mut iter = contiguous_rows_mut(&mut columns_c).unwrap();
        let row0 = iter.next().unwrap();
        assert_eq!(row0, [0.0, -0.0]);
        let row1 = iter.next().unwrap();
        row1[0] = 99.9;
        let row2 = iter.next().unwrap();
        assert_eq!(row2, [0.2, -0.2]);
    }

    {
        let mut iter = contiguous_rows(&columns_c).unwrap();
        let row0 = iter.next().unwrap();
        assert_eq!(row0, [0.0, -0.0]);
        let row1 = iter.next().unwrap();
        assert_eq!(row1, [99.9, -0.1]);
        let row2 = iter.next().unwrap();
        assert_eq!(row2, [0.2, -0.2]);
    }

    assert!(interleaved_rows(&columns_c).is_none());
    assert!(interleaved_rows_mut(&mut columns_c).is_none());

    let data = interleaved_columns_mut(&mut columns_c).unwrap();
    data[3] = -99.9;

    let data = interleaved_columns(&columns_c).unwrap();
    assert_eq!(data, [0.0, -0.0, 99.9, -99.9, 0.2, -0.2]);

    let rows_f = columns_c.t();
    assert!(!rows_f.is_standard_layout());

    assert!(contiguous_rows(&rows_f).is_none());

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

    // Channels as columns, F memory order.
    let columns_f = Array::from_shape_fn((3, 2).f(), column_data);
    assert_eq!(columns_f, array![[0.0, -0.0], [0.1, -0.1], [0.2, -0.2]]);
    assert!(!columns_f.is_standard_layout());
}
