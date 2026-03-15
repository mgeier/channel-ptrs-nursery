//! Dealing with two-dimensional arrays (using the [`ndarray`] crate).
//!
//! When interoperating with Python code that uses NumPy arrays
//! (but maybe also in other situations?) it might be useful to represent
//! blocks of multi-channel audio as two-dimensional arrays
//! (using the type [`ndarray::ArrayRef2`] in APIs).
//!
//! TODO: explain "rows" and "columns", "frames" and "channels"
//! TODO: explain "interleaved" (means "interleaved channels";
//! "interleaved frames" doesn't have a good name ... use "non-interleaved (channels)")
//!
//! TODO: explain what transposing an array does.
//!
//! TODO: rows/columns being contiguous doesn't require the whole array to be contiguous.
//!
//! These functions don't copy the array elements. In fact, that's the whole point!
//!
//! TODO: explain how to do that, e.g. convert interleaved to non-interleaved
//!
//! # Checking for contiguous or interleaved channels
//!
//! Assuming our channels are stored in columns,
//! we can differentiate between contiguous and interleaved channels
//! like this:
//!
//! ```
//! use much::ndarray::{contiguous_columns_mut, interleaved_columns_mut};
//! use ndarray::ArrayRef2;
//!
//! fn process_columns_inplace(a: &mut ArrayRef2<f32>) {
//!     if let Some(columns) = contiguous_columns_mut(a) {
//!         for column in columns {
//!             // Process contiguous column.
//!         }
//!     } else if let Some(slice) = interleaved_columns_mut(a) {
//!         // Process interleaved colums.
//!     } else {
//!         // Error: neither contiguous nor interleaved columns.
//!     }
//!
//!     // TODO: explain behavior when one-channel signal is given, interleaved or not?
//! }
//! ```
//!
//! If our channels are stored in rows,
//! we can use the `*_rows()` and `*_rows_mut()` functions instead.

// Public re-export.
pub use ndarray;

use ndarray::{ArrayRef2, Axis};

const COLUMN_AXIS: Axis = Axis(0);
const ROW_AXIS: Axis = Axis(1);

/// If the columns are contiguous in memory, returns an iterator over them.
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

/// If the columns are contiguous in memory, returns an iterator over them.
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

/// If the rows are contiguous in memory, returns an iterator over them.
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

/// If the rows are contiguous in memory, returns an iterator over them.
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

/// If the columns are interleaved, returns them as a slice.
pub fn interleaved_columns<T>(a: &ArrayRef2<T>) -> Option<&[T]> {
    if a.stride_of(ROW_AXIS) != 1 {
        return None;
    }
    a.as_slice_memory_order()
}

/// If the columns are interleaved, returns them as a slice.
pub fn interleaved_columns_mut<T>(a: &mut ArrayRef2<T>) -> Option<&mut [T]> {
    if a.stride_of(ROW_AXIS) != 1 {
        return None;
    }
    a.as_slice_memory_order_mut()
}

/// If the rows are interleaved, returns them as a slice.
pub fn interleaved_rows<T>(a: &ArrayRef2<T>) -> Option<&[T]> {
    if a.stride_of(COLUMN_AXIS) != 1 {
        return None;
    }
    a.as_slice_memory_order()
}

/// If the rows are interleaved, returns them as a slice.
pub fn interleaved_rows_mut<T>(a: &mut ArrayRef2<T>) -> Option<&mut [T]> {
    if a.stride_of(COLUMN_AXIS) != 1 {
        return None;
    }
    a.as_slice_memory_order_mut()
}
