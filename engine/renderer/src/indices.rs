pub trait Indices {
    fn id(&self) -> u32;
    fn count(&self) -> usize;
}

pub struct EmptyIndices;

impl Indices for EmptyIndices {
    fn id(&self) -> u32 {
        0
    }

    fn count(&self) -> usize {
        0
    }
}
