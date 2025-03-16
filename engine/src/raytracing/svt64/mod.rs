// From https://dubiousconst282.github.io/2024/10/03/voxel-ray-tracing/
struct SvtNode64 {
    ///  1 bit for if it is a leaf
    ///  31 for an absolute child ptr
    leaf_ptr: u32,

    /// Indicates which children are present in the array
    child_mask: u64,
}

impl SvtNode64 {
    pub fn is_leaf(&self) -> bool {
        self.leaf_ptr & 1 == 1
    }

    pub fn child_ptr(&self) -> u32 {
        self.leaf_ptr >> 1
    }
}
