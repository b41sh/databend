use crate::arrow::array::{Array, NullArray};

#[inline]
pub(super) fn equal(lhs: &NullArray, rhs: &NullArray) -> bool {
    lhs.len() == rhs.len()
}
