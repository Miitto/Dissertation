pub trait UniformValue: std::fmt::Debug {
    fn set_uniform(&self, location: usize);
}

macro_rules! impl_uniform_value {
    ($ty:ty, $method:ident, $sel:ident, $($expr:expr),+ ) => {
        impl UniformValue for $ty {
            fn set_uniform(&$sel, location: usize) {
                unsafe { gl::$method(location as i32, $($expr),+) };
            }
        }
    };
}

impl_uniform_value!(i32, Uniform1i, self, *self);
impl_uniform_value!([i32; 2], Uniform2i, self, self[0], self[1]);
impl_uniform_value!([i32; 3], Uniform3i, self, self[0], self[1], self[2]);
impl_uniform_value!(
    [i32; 4], Uniform4i, self, self[0], self[1], self[2], self[3]
);

impl_uniform_value!(u32, Uniform1ui, self, *self);
impl_uniform_value!([u32; 2], Uniform2ui, self, self[0], self[1]);
impl_uniform_value!([u32; 3], Uniform3ui, self, self[0], self[1], self[2]);
impl_uniform_value!(
    [u32; 4], Uniform4ui, self, self[0], self[1], self[2], self[3]
);

impl_uniform_value!(f32, Uniform1f, self, *self);
impl_uniform_value!([f32; 2], Uniform2f, self, self[0], self[1]);
impl_uniform_value!([f32; 3], Uniform3f, self, self[0], self[1], self[2]);
impl_uniform_value!(
    [f32; 4], Uniform4f, self, self[0], self[1], self[2], self[3]
);

impl_uniform_value!(
    [f32; 16],
    UniformMatrix4fv,
    self,
    1,
    gl::FALSE,
    self.as_ptr()
);

impl_uniform_value!(
    [[f32; 4]; 4],
    UniformMatrix4fv,
    self,
    1,
    gl::FALSE,
    self.as_ptr() as *const f32
);

impl<T> UniformValue for Option<T>
where
    T: UniformValue,
{
    fn set_uniform(&self, location: usize) {
        if let Some(val) = self {
            val.set_uniform(location);
        }
    }
}
