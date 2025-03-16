use std::marker::PhantomData;

use render_common::Program;

use crate::{LayoutBlock, SSBO, SSBOBlock, UniformBlock, Uniforms};

use super::{Buffer, BufferError, BufferMode, FencedRawBuffer};

pub struct ShaderBuffer<U>
where
    U: LayoutBlock,
{
    buffer: FencedRawBuffer,
    phantom: PhantomData<U>,
}

impl<U> ShaderBuffer<U>
where
    U: LayoutBlock,
{
    pub fn new(uniforms: U) -> Result<Self, BufferError> {
        // println!("Creating Uniform buffer with size: {}", U::size());
        let mut buffer = FencedRawBuffer::empty(U::size(), BufferMode::Persistent)?;

        uniforms.set_buffer_data(&mut buffer)?;

        Ok(Self {
            buffer,
            phantom: PhantomData,
        })
    }

    pub fn id(&self) -> u32 {
        self.buffer.id()
    }

    pub fn set_data(&mut self, data: &U) -> Result<(), BufferError> {
        data.set_buffer_data(&mut self.buffer)
    }
}

impl<T> ShaderBuffer<T>
where
    T: LayoutBlock,
{
    pub fn bind_to(&self, location: gl::types::GLenum) {
        unsafe {
            gl::BindBufferBase(location, T::bind_point(), self.buffer.id());
        }
    }
}

impl<U> ShaderBuffer<U>
where
    U: LayoutBlock + UniformBlock,
{
    pub fn bind(&self) {
        self.bind_to(gl::UNIFORM_BUFFER);
    }
}

impl<B> SSBO for ShaderBuffer<B>
where
    B: LayoutBlock + SSBOBlock,
{
    fn bind(&self) {
        self.bind_to(gl::SHADER_STORAGE_BUFFER);
    }
}
