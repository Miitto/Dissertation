use std::marker::PhantomData;

use crate::Program;

use crate::{LayoutBlock, SSBO, SSBOBlock, UniformBlock, Uniforms};

use super::{Buffer, BufferError, BufferMode, FencedRawBuffer, RawBuffer};

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
    pub fn new(uniforms: &[U]) -> Result<Self, BufferError> {
        // println!("Creating Uniform buffer with size: {}", U::size());

        let size = uniforms.iter().map(|u| u.size()).sum::<usize>();
        let mut buffer = FencedRawBuffer::empty(size, BufferMode::Persistent)?;

        if !uniforms.is_empty() {
            let mut mapping = buffer.get_mapping();

            let mut offset = 0;
            for uniform in uniforms {
                offset = uniform.set_buffer_data(&mut mapping, offset)?;
            }
        }

        Ok(Self {
            buffer,
            phantom: PhantomData,
        })
    }

    pub fn single(uniform: &U) -> Result<Self, BufferError> {
        let mut buffer = FencedRawBuffer::empty(uniform.size(), BufferMode::Persistent)?;

        {
            let mut mapping = buffer.get_mapping();
            uniform.set_buffer_data(&mut mapping, 0)?;
        }

        Ok(Self {
            buffer,
            phantom: PhantomData,
        })
    }

    pub fn id(&self) -> u32 {
        self.buffer.id()
    }

    pub fn set_single(&mut self, uniform: &U, offset: usize) -> Result<(), BufferError> {
        let size = uniform.size();
        if size + offset > self.buffer.size() {
            let buffer = FencedRawBuffer::empty(size + offset, BufferMode::Persistent)
                .expect("Failed to make larger ShaderBuffer");
            if offset != 0 {
                if let Err(e) = self.buffer.copy_to(&buffer, 0, 0, offset) {
                    eprintln!("Error copying over old data: {:?}", e);
                }
            }

            self.buffer = buffer;
        }

        let mut mapping = self.buffer.get_mapping();
        uniform.set_buffer_data(&mut mapping, offset)?;

        Ok(())
    }

    pub fn set_data(&mut self, data: &[U], mut offset: usize) -> Result<usize, BufferError> {
        crate::profiler::event!("Setting SSBO data");
        let size = data.iter().map(|u| u.size()).sum::<usize>();

        let total_size = size + offset;

        if total_size == 0 {
            return Ok(offset);
        }

        if size + offset > self.buffer.size() {
            let buffer = FencedRawBuffer::empty(total_size, BufferMode::Persistent)
                .expect("Failed to make larger ShaderBuffer");
            if offset != 0 {
                if let Err(e) = self.buffer.copy_to(&buffer, 0, 0, offset) {
                    eprintln!("Error copying over old data: {:?}", e);
                }
            }

            self.buffer = buffer;
        }

        let mut mapping = self.buffer.get_mapping();

        for uniform in data {
            offset = uniform.set_buffer_data(&mut mapping, offset)?;
        }

        Ok(offset)
    }

    pub fn set_label(&mut self, label: &str) {
        self.buffer.set_label(label);
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

impl<U> Uniforms for ShaderBuffer<U>
where
    U: LayoutBlock + UniformBlock,
{
    fn bind(&self, _: &Program) {
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
