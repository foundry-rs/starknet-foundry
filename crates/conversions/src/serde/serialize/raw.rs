use super::{BufferWriter, CairoSerialize};

/// use this wrapper to NOT add extra length felt
/// useful e.g. when you need to pass an already serialized value
pub struct RawFeltVec<T>(pub(crate) Vec<T>)
where
    T: CairoSerialize;

impl<T> RawFeltVec<T>
where
    T: CairoSerialize,
{
    #[must_use]
    pub fn new(vec: Vec<T>) -> Self {
        Self(vec)
    }
}

impl<T> CairoSerialize for RawFeltVec<T>
where
    T: CairoSerialize,
{
    fn serialize(&self, output: &mut BufferWriter) {
        for e in &self.0 {
            e.serialize(output);
        }
    }
}
