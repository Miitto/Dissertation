use super::{Buffer, BufferError, BufferMode, Mapping, RawBuffer};

pub struct GpuBuffer {
    id: gl::types::GLuint,
    count: usize,
    size: usize,
    mapping: Option<*mut std::os::raw::c_void>,
    mode: BufferMode,
}

impl GpuBuffer {
    pub fn reallocate_with_size(&mut self, size: usize) -> Result<(), BufferError> {
        println!("Reallocating GpuBuffer");
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }

        self.size = size;
        self.id = 0;

        self.create_with(std::ptr::null())?;

        Ok(())
    }

    fn create_with(
        &mut self,
        data: *const std::os::raw::c_void,
    ) -> std::result::Result<&mut Self, BufferError> {
        // println!(
        //     "Creating Buffer with size {} and length: {}",
        //     self.size, self.count
        // );
        unsafe {
            gl::CreateBuffers(1, &mut self.id);
            gl::NamedBufferStorage(
                self.id,
                // NVidia drivers error with size of 0
                (self.size.max(1)) as isize,
                data,
                self.mode.to_buf_store(),
            );
        }

        let mapping = if let BufferMode::Persistent = self.mode {
            Some(unsafe {
                gl::MapNamedBufferRange(self.id, 0, self.size as isize, self.mode.to_buf_store())
            })
        } else {
            None
        };

        self.mapping = mapping;

        Ok(self)
    }
}

impl Buffer for GpuBuffer {
    fn buf_mode(&self) -> BufferMode {
        self.mode
    }

    fn count(&self) -> usize {
        self.count
    }

    fn size(&self) -> usize {
        self.size
    }

    fn id(&self) -> gl::types::GLuint {
        self.id
    }

    fn immutable(&self) -> bool {
        matches!(self.mode, BufferMode::Immutable | BufferMode::Persistent)
    }

    fn empty(size: usize, mode: BufferMode) -> Result<Self, BufferError> {
        // println!("Creating empty GpuBuffer");
        let mut buf = Self {
            id: 0,
            size,
            count: 0,
            mapping: None,
            mode,
        };

        buf.create_with(std::ptr::null())?;

        Ok(buf)
    }

    fn with_data<T>(data: &[T], mode: BufferMode) -> Result<Self, BufferError>
    where
        T: Sized,
    {
        // println!("Creating GpuBuffer with data");
        let size = std::mem::size_of_val(data);

        let mut buf = Self {
            id: 0,
            size,
            count: data.len(),
            mapping: None,
            mode,
        };

        buf.create_with(data.as_ptr() as *const std::os::raw::c_void)?;

        Ok(buf)
    }

    fn set_data<T>(&mut self, data: &[T]) -> Result<bool, BufferError>
    where
        T: Sized,
    {
        self.set_offset_data(0, data)
    }
}

impl RawBuffer for GpuBuffer {
    fn set_offset_data<T>(&mut self, offset: usize, data: &[T]) -> Result<bool, BufferError>
    where
        T: Sized,
    {
        let size = std::mem::size_of_val(data);

        let realloc = if size + offset > self.size() {
            self.reallocate_with_size(size)?;
            true
        } else {
            false
        };

        self.set_offset_data_no_alloc(offset, data)?;
        Ok(realloc)
    }

    fn set_data_no_alloc<T>(&mut self, data: &[T]) -> Result<(), BufferError>
    where
        T: Sized,
    {
        self.set_offset_data_no_alloc(0, data)
    }

    fn set_offset_data_no_alloc<T>(&mut self, offset: usize, data: &[T]) -> Result<(), BufferError>
    where
        T: Sized,
    {
        let size = std::mem::size_of_val(data);
        if size + offset > self.size() {
            panic!(
                "Attempted to write outside of buffer bounds | Size: {}, Offset: {}, Buffer Size: {}",
                size,
                offset,
                self.size()
            );
        }
        assert!(size > 0);

        if let Some(mapping) = self.mapping {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    data.as_ptr() as *const u8,
                    mapping.add(offset) as *mut u8,
                    size,
                );
            }

            self.count = data.len();

            return Ok(());
        }

        if !self.immutable() {
            unsafe {
                gl::NamedBufferSubData(
                    self.id,
                    offset as isize,
                    size as isize,
                    data.as_ptr() as *const std::ffi::c_void,
                );
            }
        } else {
            // println!("Creating Copy Buffer");
            let mut copy_buf = GpuBuffer::with_data(data, BufferMode::Immutable)?;
            copy_buf.copy_to(self, 0, offset, size)?;
        }

        let mut size = 0;
        unsafe {
            gl::GetNamedBufferParameteriv(self.id, gl::BUFFER_SIZE, &mut size);
        }

        if size as usize != self.size() {
            return Err(BufferError::OutOfMemory);
        }

        self.count = data.len();

        Ok(())
    }

    fn copy_to<T: RawBuffer>(
        &mut self,
        other: &T,
        src_offset: usize,
        dst_offset: usize,
        size: usize,
    ) -> Result<(), BufferError> {
        if self.size < size + src_offset || other.size() < size + dst_offset {
            return Err(BufferError::InvalidSize);
        }

        unsafe {
            gl::CopyNamedBufferSubData(
                self.id,
                other.id(),
                src_offset as isize,
                dst_offset as isize,
                size as isize,
            );
        }

        Ok(())
    }

    fn get_mapping<'a>(&'a mut self) -> super::Mapping<'a> {
        if let Some(mapping) = self.mapping {
            Mapping::new(self, mapping, self.size)
        } else {
            panic!(
                "Buffer has no mapping: Buffer Type: {:?} | Size: {}",
                self.buf_mode(),
                self.size()
            );
        }
    }

    fn on_map_flush(&mut self) {}
}

impl Drop for GpuBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}
