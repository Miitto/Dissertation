use crate::indices::Indices;

use super::{Buffer, BufferError, BufferMode, BufferType, GpuBuffer};

#[allow(dead_code)]
pub struct Ebo {
    buffer: GpuBuffer,
    len: usize,
}

impl Ebo {
    pub fn bind(&self) {
        self.buffer.bind(BufferType::ElementArrayBuffer);
    }

    pub fn unbind() {
        unsafe { gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0) };
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Indices for Ebo {
    fn id(&self) -> u32 {
        self.buffer.id()
    }

    fn count(&self) -> usize {
        self.len
    }
}

impl Buffer for Ebo {
    fn buf_mode(&self) -> BufferMode {
        self.buffer.buf_mode()
    }

    fn count(&self) -> usize {
        self.buffer.count()
    }

    fn size(&self) -> usize {
        self.buffer.size()
    }

    fn id(&self) -> gl::types::GLuint {
        self.buffer.id()
    }

    fn immutable(&self) -> bool {
        self.buffer.immutable()
    }

    fn with_data<T>(data: &[T], mode: BufferMode) -> Result<Self, BufferError>
    where
        T: Sized,
    {
        // println!("Creating EBO with data");
        let buffer = GpuBuffer::with_data(data, mode)?;

        Ok(Self {
            buffer,
            len: data.len(),
        })
    }

    fn empty(size: usize, mode: BufferMode) -> Result<Self, BufferError> {
        // println!("Creating empty EBO");
        let buffer = GpuBuffer::empty(size, mode)?;

        Ok(Self { buffer, len: 0 })
    }

    fn set_data<T>(&mut self, data: &[T]) -> Result<bool, BufferError>
    where
        T: Sized,
    {
        let realloc = self.buffer.set_data(data)?;
        self.len = data.len();
        Ok(realloc)
    }

    fn set_label(&mut self, label: impl Into<String>) {
        self.buffer.set_label(label);
    }
}
