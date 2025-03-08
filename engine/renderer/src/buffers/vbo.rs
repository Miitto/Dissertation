use std::marker::PhantomData;

use render_common::format::VertexFormat;

use crate::vertex::Vertex;

use super::{Buffer, BufferError, BufferMode, BufferType, FencedBuffer, FencedRawBuffer};

pub struct Vbo<T>
where
    T: Vertex,
{
    buffer: FencedRawBuffer,
    len: usize,
    phantom: PhantomData<T>,
}

impl<T> Vbo<T>
where
    T: Vertex,
{
    pub fn get_format(&self) -> (u32, VertexFormat) {
        let bindings = T::bindings();

        let stride = std::mem::size_of::<T>();

        (stride as u32, bindings)
    }

    pub fn bind(&self) {
        self.buffer.bind(BufferType::ArrayBuffer);
    }

    pub fn unbind() {
        unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, 0) };
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn updating(&self) -> bool {
        !self.buffer.signalled()
    }
}

impl<T> Buffer for Vbo<T>
where
    T: Vertex,
{
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

    fn with_data<D: Sized>(data: &[D], mode: BufferMode) -> Result<Self, BufferError> {
        println!("Creating VBO with data");
        let buffer = FencedRawBuffer::with_data(data, mode)?;

        Ok(Self {
            buffer,
            len: data.len(),
            phantom: PhantomData,
        })
    }

    fn empty(size: usize, mode: BufferMode) -> Result<Self, BufferError> {
        println!("Creating empty VBO");
        let buffer = FencedRawBuffer::empty(size, mode)?;

        Ok(Self {
            buffer,
            len: 0,
            phantom: PhantomData,
        })
    }

    fn set_data<D>(&mut self, data: &[D]) -> Result<bool, BufferError>
    where
        D: Sized,
    {
        let realloc = self.buffer.set_data(data)?;
        self.len = data.len();

        Ok(realloc)
    }
}

impl<T> FencedBuffer for Vbo<T>
where
    T: Vertex,
{
    type Buffer = FencedRawBuffer;

    fn buffer(&self) -> &Self::Buffer {
        &self.buffer
    }

    fn buffer_mut(&mut self) -> &mut Self::Buffer {
        &mut self.buffer
    }

    fn from_buffer(buffer: Self::Buffer) -> Self {
        Self {
            buffer,
            len: 0,
            phantom: PhantomData,
        }
    }

    fn signalled(&self) -> bool {
        self.buffer.signalled()
    }

    fn start_fence(&mut self) {
        self.buffer.start_fence();
    }
}
