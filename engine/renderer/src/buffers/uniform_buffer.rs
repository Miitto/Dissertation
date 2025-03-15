use std::marker::PhantomData;

use render_common::Program;

use crate::{LayoutBlock, Uniforms};

use super::{Buffer, BufferError, BufferMode, FencedRawBuffer};

pub struct UniformBuffer<U>
where
    U: LayoutBlock,
{
    buffer: FencedRawBuffer,
    phantom: PhantomData<U>,
}

impl<U> UniformBuffer<U>
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

impl<U> Uniforms for UniformBuffer<U>
where
    U: LayoutBlock,
{
    fn bind(&self, _program: &Program) {
        unsafe { gl::BindBufferBase(gl::UNIFORM_BUFFER, U::bind_point(), self.buffer.id()) }
    }
}
