use std::marker::PhantomData;

use crate::{
    DrawMode,
    indices::Indices,
    vertex::{Vertex, format::VertexFormat},
};

use super::{Buffer, Vbo};

#[allow(dead_code)]
pub struct Vao<T, I = T>
where
    T: Vertex,
    I: Vertex,
{
    id: gl::types::GLuint,
    pub mode: DrawMode,
    len: usize,
    indice_count: usize,
    instance_count: usize,
    vbo: PhantomData<T>,
    instance_vbo: PhantomData<I>,
}

#[derive(Debug, Copy, Clone)]
pub struct EmptyVertex;

impl Vertex for EmptyVertex {
    fn bindings() -> VertexFormat {
        &[]
    }
}

impl<T, I> Vao<T, I>
where
    T: Vertex,
    I: Vertex,
{
    pub fn new(mode: DrawMode) -> Self
    where
        T: Vertex,
    {
        let mut id = 0;
        unsafe {
            gl::CreateVertexArrays(1, &mut id);
        }

        Self {
            id,
            len: 0,
            indice_count: 0,
            instance_count: 0,
            mode,
            vbo: PhantomData,
            instance_vbo: PhantomData,
        }
    }

    pub fn draw_mode(&self) -> DrawMode {
        self.mode
    }

    pub fn setup_vbo(
        &self,
        id: u32,
        stride: usize,
        bind_point: u32,
        instance: bool,
        bindings: VertexFormat,
    ) {
        unsafe { gl::VertexArrayVertexBuffer(self.id, bind_point, id, 0, stride as i32) }

        for binding in bindings {
            unsafe {
                gl::EnableVertexArrayAttrib(self.id, binding.location);
                if binding.is_int {
                    gl::VertexArrayAttribIFormat(
                        self.id,
                        binding.location,
                        binding.elements,
                        binding.ty,
                        binding.offset,
                    );
                } else {
                    gl::VertexArrayAttribFormat(
                        self.id,
                        binding.location,
                        binding.elements,
                        binding.ty,
                        gl::FALSE,
                        binding.offset,
                    );
                }

                gl::VertexArrayAttribBinding(self.id, binding.location, bind_point);

                if instance {
                    gl::VertexArrayBindingDivisor(self.id, bind_point, 1);
                }
            }
        }
    }

    pub fn set_vert_vbo(&mut self, id: u32) {
        unsafe { gl::VertexArrayVertexBuffer(self.id, 0, id, 0, std::mem::size_of::<T>() as i32) }
    }

    pub fn set_instance_vbo(&mut self, id: u32) {
        unsafe {
            gl::VertexArrayVertexBuffer(self.id, 1, id, 0, std::mem::size_of::<T>() as i32);
        }
    }

    pub fn setup_vertices(&self, vbo: &Vbo<T>) {
        let stride = std::mem::size_of::<T>();
        let bindings = T::bindings();

        self.setup_vbo(vbo.id(), stride, 0, false, bindings);
    }

    pub fn setup_instances(&self, vbo: &Vbo<I>) {
        let stride = std::mem::size_of::<I>();
        let bindings = I::bindings();

        self.setup_vbo(vbo.id(), stride, 1, true, bindings);
    }

    pub fn setup_indices(&self, indices: &impl Indices) {
        unsafe {
            gl::VertexArrayElementBuffer(self.id, indices.id());
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        };
    }

    pub fn unbind() {
        unsafe {
            gl::BindVertexArray(0);
        };
    }

    pub fn unbind_all(&self) {
        unsafe {
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        };
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn has_indices(&self) -> bool {
        self.indice_count > 0
    }

    pub fn indice_count(&self) -> i32 {
        self.indice_count as i32
    }

    pub fn instanced(&self) -> bool {
        self.instance_count > 0
    }

    pub fn instance_count(&self) -> i32 {
        self.instance_count as i32
    }
}

impl<T, I> Drop for Vao<T, I>
where
    T: Vertex,
    I: Vertex,
{
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.id);
        };
    }
}
