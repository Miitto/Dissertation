use std::marker::PhantomData;

use shaders::Program;

use crate::Uniforms;

pub struct Material<P, U>
where
    P: Program,
    U: Uniforms,
{
    pub program: PhantomData<P>,
    pub uniforms: U,
}
