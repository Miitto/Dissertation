use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

use super::{Buffer, BufferError, BufferMode, Mapping, RawBuffer};

pub struct Pointer<T> {
    pub ptr: *mut T,
}

impl Deref for Pointer<std::os::raw::c_void> {
    type Target = *mut std::os::raw::c_void;

    fn deref(&self) -> &Self::Target {
        &self.ptr
    }
}

unsafe impl Send for Pointer<std::os::raw::c_void> {}

#[derive(Clone)]
pub struct MappingAddr {
    pub ptr: Arc<Mutex<Pointer<std::os::raw::c_void>>>,
}

impl Deref for MappingAddr {
    type Target = Arc<Mutex<Pointer<std::os::raw::c_void>>>;

    fn deref(&self) -> &Self::Target {
        &self.ptr
    }
}

pub struct GpuBuffer {
    id: gl::types::GLuint,
    count: usize,
    size: usize,
    mapping: Option<MappingAddr>,
    mode: BufferMode,
    label: Option<String>,
}

impl GpuBuffer {
    pub fn reallocate_with_size(&mut self, size: usize) -> Result<(), BufferError> {
        println!(
            "Reallocating GpuBuffer: {}",
            self.label.clone().unwrap_or_default()
        );
        let old_size = self.size;
        let old_id = self.id;

        self.size = size;
        self.id = 0;

        self.create_with(std::ptr::null())?;

        unsafe {
            gl::CopyNamedBufferSubData(old_id, self.id, 0, 0, old_size as isize);
        }

        unsafe {
            gl::DeleteBuffers(1, &old_id);
        }

        Ok(())
    }

    fn create_with(
        &mut self,
        data: *const std::os::raw::c_void,
    ) -> std::result::Result<&mut Self, BufferError> {
        crate::profiler::event!("Creating buffer");
        if self.size == 0 {
            return Ok(self);
        }
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

        let mapping = if matches!(
            self.mode,
            BufferMode::Persistent | BufferMode::PersistentCoherent
        ) {
            let map = match self.buf_mode() {
                BufferMode::Persistent => {
                    self.buf_mode().to_buf_store() | gl::MAP_FLUSH_EXPLICIT_BIT
                }
                BufferMode::PersistentCoherent => self.buf_mode().to_buf_store(),
                _ => unreachable!(),
            };
            Some(unsafe { gl::MapNamedBufferRange(self.id, 0, self.size as isize, map) })
        } else {
            None
        };

        self.mapping = mapping.map(|ptr| MappingAddr {
            ptr: Arc::new(Mutex::new(Pointer { ptr })),
        });

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
        matches!(
            self.mode,
            BufferMode::Immutable | BufferMode::Persistent | BufferMode::PersistentCoherent
        )
    }

    fn empty(size: usize, mode: BufferMode) -> Result<Self, BufferError> {
        // println!("Creating empty GpuBuffer");
        let mut buf = Self {
            id: 0,
            size,
            count: 0,
            mapping: None,
            mode,
            label: None,
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
            label: None,
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

    fn set_label(&mut self, label: impl Into<String>) {
        self.label = Some(label.into());
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
        crate::profiler::event!("Buffer write");
        let size = std::mem::size_of_val(data);
        if size + offset > self.size() {
            panic!(
                "Attempted to write outside of buffer bounds: {} | Size: {}, Offset: {}, Buffer Size: {}",
                self.label.clone().unwrap_or_default(),
                size,
                offset,
                self.size()
            );
        }

        // Nothing to write, and will error if size is 0 (at least on NVidia)
        if size == 0 {
            return Ok(());
        }

        if self.mapping.is_some() {
            {
                let mut mapping = self.get_mapping();

                unsafe {
                    mapping.write(data.as_ptr() as *const u8, size, offset);
                }
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
        crate::profiler::event!("Buffer Copy");
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

    fn raw_mapping(&self) -> Option<MappingAddr> {
        self.mapping.clone()
    }

    fn get_mapping<'a>(&'a mut self) -> super::Mapping<'a, GpuBuffer> {
        if let Some(ref mapping) = self.mapping {
            Mapping::new(
                self,
                mapping.clone(),
                self.size,
                matches!(self.buf_mode(), BufferMode::PersistentCoherent),
            )
        } else {
            panic!(
                "Buffer has no mapping: {} | Buffer Type: {:?} | Size: {}",
                self.label.clone().unwrap_or_default(),
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
