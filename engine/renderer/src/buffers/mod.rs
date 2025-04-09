mod ebo;
mod fenced_buffer;
mod gpu_buffer;
mod mapping;
mod ssbo;
mod vao;
mod vbo;

pub use ebo::*;
pub use fenced_buffer::*;
pub use gpu_buffer::*;
pub use mapping::*;
pub use ssbo::*;
pub use vao::*;
pub use vbo::*;

#[derive(Debug)]
pub enum BufferError {
    InvalidSize,
    OutOfMemory,
}

pub trait Buffer: Sized {
    fn buf_mode(&self) -> BufferMode;
    fn with_data<T>(data: &[T], mode: BufferMode) -> Result<Self, BufferError>
    where
        T: Sized;
    fn empty(size: usize, mode: BufferMode) -> Result<Self, BufferError>;
    fn count(&self) -> usize;
    fn size(&self) -> usize;
    fn id(&self) -> gl::types::GLuint;
    fn immutable(&self) -> bool;
    fn bind(&self, ty: BufferType) {
        unsafe {
            gl::BindBuffer(ty.to_gl(), self.id());
        }
    }
    fn set_data<T>(&mut self, data: &[T]) -> Result<bool, BufferError>
    where
        T: Sized;
}

pub trait RawBuffer: Buffer {
    fn set_offset_data<T>(&mut self, offset: usize, data: &[T]) -> Result<bool, BufferError>
    where
        T: Sized;
    fn set_data_no_alloc<T>(&mut self, data: &[T]) -> Result<(), BufferError>
    where
        T: Sized;
    fn set_offset_data_no_alloc<T>(&mut self, offset: usize, data: &[T]) -> Result<(), BufferError>
    where
        T: Sized;
    fn copy_to<T: RawBuffer>(
        &mut self,
        other: &T,
        src_offset: usize,
        dst_offset: usize,
        size: usize,
    ) -> Result<(), BufferError>;
    fn raw_mapping(&self) -> Option<MappingAddr>;
    fn get_mapping<'a>(&'a mut self) -> Mapping<'a, Self>;
    fn on_map_flush(&mut self);
}

#[derive(Debug, Clone, Copy)]
pub enum BufferType {
    ArrayBuffer,
    UniformBuffer,
    ElementArrayBuffer,
}

impl BufferType {
    pub fn to_gl(&self) -> gl::types::GLenum {
        match self {
            BufferType::ArrayBuffer => gl::ARRAY_BUFFER,
            BufferType::UniformBuffer => gl::UNIFORM_BUFFER,
            BufferType::ElementArrayBuffer => gl::ELEMENT_ARRAY_BUFFER,
        }
    }
}

impl From<BufferType> for gl::types::GLenum {
    fn from(ty: BufferType) -> Self {
        ty.to_gl()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BufferMode {
    Default,
    Immutable,
    Dynamic,
    Persistent,
    PersistentCoherent,
}

impl BufferMode {
    pub fn to_buf_data(&self) -> gl::types::GLenum {
        match self {
            BufferMode::Default | BufferMode::Immutable => gl::STATIC_DRAW,
            BufferMode::Dynamic | BufferMode::Persistent | BufferMode::PersistentCoherent => {
                gl::DYNAMIC_DRAW
            }
        }
    }

    pub fn to_buf_store(&self) -> gl::types::GLenum {
        match self {
            BufferMode::Default => gl::DYNAMIC_STORAGE_BIT | gl::MAP_READ_BIT | gl::MAP_WRITE_BIT,
            BufferMode::Dynamic => {
                gl::DYNAMIC_STORAGE_BIT
                    | gl::CLIENT_STORAGE_BIT
                    | gl::MAP_READ_BIT
                    | gl::MAP_WRITE_BIT
            }
            BufferMode::Persistent => gl::MAP_READ_BIT | gl::MAP_WRITE_BIT | gl::MAP_PERSISTENT_BIT,
            BufferMode::PersistentCoherent => {
                gl::MAP_READ_BIT | gl::MAP_WRITE_BIT | gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT
            }
            BufferMode::Immutable => 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BufferCreationMode {
    // BufferData,
    BufferStorage,
}
