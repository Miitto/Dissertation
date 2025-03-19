use crate::fence::Fence;

use super::{Buffer, BufferError, GpuBuffer, RawBuffer};

pub trait FencedBuffer: Buffer {
    type Buffer: Buffer;
    fn buffer(&self) -> &Self::Buffer;
    fn buffer_mut(&mut self) -> &mut Self::Buffer;
    fn from_buffer(buffer: Self::Buffer) -> Self;
    fn signalled(&self) -> bool;
    fn start_fence(&mut self);
}

pub struct FencedRawBuffer {
    buffer: GpuBuffer,
    fence: Fence,
}

impl FencedBuffer for FencedRawBuffer {
    type Buffer = GpuBuffer;

    fn buffer(&self) -> &GpuBuffer {
        &self.buffer
    }

    fn buffer_mut(&mut self) -> &mut GpuBuffer {
        &mut self.buffer
    }

    fn from_buffer(buffer: GpuBuffer) -> Self {
        Self {
            buffer,
            fence: Fence::default(),
        }
    }

    fn signalled(&self) -> bool {
        self.fence.signalled()
    }

    fn start_fence(&mut self) {
        self.fence.start();
    }
}

impl Buffer for FencedRawBuffer {
    fn buf_mode(&self) -> super::BufferMode {
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

    fn empty(size: usize, mode: super::BufferMode) -> Result<Self, BufferError> {
        // println!("Creating empty FencedRawBuffer");
        let buffer = GpuBuffer::empty(size, mode)?;

        Ok(Self {
            buffer,
            fence: Fence::default(),
        })
    }

    fn with_data<T: Sized>(data: &[T], mode: super::BufferMode) -> Result<Self, BufferError> {
        // println!("Creating FencedRawBuffer with data");
        let buffer = GpuBuffer::with_data(data, mode)?;

        Ok(Self {
            buffer,
            fence: Fence::default(),
        })
    }

    fn set_data<T>(&mut self, data: &[T]) -> Result<bool, BufferError>
    where
        T: Sized,
    {
        let realloc = self.buffer.set_data(data)?;

        self.fence.start();

        Ok(realloc)
    }
}

impl RawBuffer for FencedRawBuffer {
    fn set_offset_data<T>(&mut self, offset: usize, data: &[T]) -> Result<bool, BufferError>
    where
        T: Sized,
    {
        let realloc = self.buffer.set_offset_data(offset, data)?;

        self.fence.start();

        Ok(realloc)
    }

    fn set_data_no_alloc<T>(&mut self, data: &[T]) -> Result<(), BufferError>
    where
        T: Sized,
    {
        self.buffer.set_data_no_alloc(data)?;
        self.fence.start();
        Ok(())
    }

    fn set_offset_data_no_alloc<T>(&mut self, offset: usize, data: &[T]) -> Result<(), BufferError>
    where
        T: Sized,
    {
        self.buffer.set_offset_data_no_alloc(offset, data)?;
        self.fence.start();
        Ok(())
    }

    fn copy_to<T: RawBuffer>(
        &mut self,
        other: &T,
        src_offset: usize,
        dst_offset: usize,
        size: usize,
    ) -> Result<(), BufferError> {
        self.buffer.copy_to(other, src_offset, dst_offset, size)?;

        self.fence.start();

        Ok(())
    }

    fn get_mapping<'a>(&'a mut self) -> super::Mapping<'a> {
        self.buffer.get_mapping()
    }

    fn on_map_flush(&mut self) {
        self.buffer.on_map_flush();

        self.fence.start();
    }
}
