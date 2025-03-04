use std::marker::PhantomData;

use crate::{
    DrawMode, DrawType,
    vertex::{Vertex, format::VertexFormat},
};

#[allow(dead_code)]
pub struct Vao<T, I = EmptyVertex>
where
    T: Vertex,
    I: Vertex,
{
    id: Option<gl::types::GLuint>,
    vbo: Option<Vbo<T>>,
    instance_vbo: Option<Vbo<I>>,
    indices: Option<Indices>,
    ty: DrawType,
    pub mode: DrawMode,
}

#[derive(Debug, Copy, Clone)]
pub struct EmptyVertex;

impl Vertex for EmptyVertex {
    fn bindings() -> VertexFormat {
        &[]
    }
}

impl<T> Vao<T>
where
    T: Vertex,
{
    pub fn new(data: &[T], indices: Option<&[u32]>, ty: DrawType, mode: DrawMode) -> Self
    where
        T: Vertex,
    {
        Vao::new_typed::<EmptyVertex>(data, indices, ty, mode)
    }

    pub fn new_typed<I>(
        data: &[T],
        indices: Option<&[u32]>,
        ty: DrawType,
        mode: DrawMode,
    ) -> Vao<T, I>
    where
        T: Vertex,
        I: Vertex,
    {
        let vbo = Vbo::new(data, ty, false);

        let mut vao = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);

            gl::BindVertexArray(vao);
        };

        let indices = if let Some(indices) = indices {
            let ebo = Ebo::new(indices, ty);
            Indices::Ebo(ebo)
        } else {
            Indices::None
        };

        vbo.setup();

        Vao {
            id: Some(vao),
            ty,
            vbo: Some(vbo),
            instance_vbo: None,
            indices: Some(indices),
            mode,
        }
    }
}

impl<T, I> Vao<T, I>
where
    T: Vertex,
    I: Vertex,
{
    pub fn new_instanced(
        data: &[T],
        indices: Option<&[u32]>,
        ty: DrawType,
        mode: DrawMode,
        instance: &[I],
    ) -> Self
    where
        T: Vertex,
        I: Vertex,
    {
        let instance_vbo = Vbo::new(instance, ty, true);

        let mut vao = Vao::new(data, indices, ty, mode);

        instance_vbo.setup();

        Vao {
            id: vao.id.take(),
            ty: vao.ty,
            vbo: vao.vbo.take(),
            instance_vbo: Some(instance_vbo),
            indices: vao.indices.take(),
            mode: vao.mode,
        }
    }

    pub fn setup(&self) {
        self.bind();

        if let Some(vbo) = &self.vbo {
            vbo.setup();
        }

        if let Some(instance_vbo) = &self.instance_vbo {
            instance_vbo.setup();
        }

        if let Some(indices) = &self.indices {
            indices.bind();
        }

        self.unbind_all();
    }

    pub fn new_maybe_instanced(
        data: &[T],
        indices: Option<&[u32]>,
        ty: DrawType,
        mode: DrawMode,
        instance: Option<&[I]>,
    ) -> Self
    where
        T: Vertex,
        I: Vertex,
    {
        if let Some(instance) = instance {
            Vao::new_instanced(data, indices, ty, mode, instance)
        } else {
            Vao::new_typed::<I>(data, indices, ty, mode)
        }
    }

    pub fn update_instances(&mut self, data: &[I]) {
        if let Some(instance_vbo) = &mut self.instance_vbo {
            instance_vbo.bind();
            instance_vbo.update(data);

            self.bind();
            self.setup();
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id.unwrap());
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
        if let Indices::Ebo(ebo) = &self.indices.as_ref().unwrap() {
            ebo.len
        } else {
            self.vbo.as_ref().unwrap().len
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn has_indices(&self) -> bool {
        !matches!(&self.indices.as_ref().unwrap(), Indices::None)
    }

    pub fn instanced(&self) -> bool {
        self.instance_vbo.is_some()
    }

    pub fn instance_count(&self) -> i32 {
        self.instance_vbo.as_ref().map(|i| i.len).unwrap_or(0) as i32
    }
}

impl<T, I> Drop for Vao<T, I>
where
    T: Vertex,
    I: Vertex,
{
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            unsafe {
                gl::DeleteVertexArrays(1, &id);
            };
        }
    }
}

pub struct Vbo<T>
where
    T: Vertex,
{
    id: gl::types::GLuint,
    len: usize,
    instance: bool,
    data: PhantomData<T>,
    ty: DrawType,
}

impl<T> Vbo<T>
where
    T: Vertex,
{
    pub fn new(data: &[T], ty: DrawType, instance: bool) -> Self {
        let mut vbo = 0;
        unsafe {
            gl::GenBuffers(1, &mut vbo);
        };

        let size = std::mem::size_of_val(data);

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size as gl::types::GLsizeiptr,
                data.as_ptr() as *const _,
                ty.into(),
            );
        };

        let vbo = Vbo {
            id: vbo,
            len: data.len(),
            instance,
            data: PhantomData,
            ty,
        };

        Vbo::<T>::unbind();

        vbo
    }

    pub fn update(&mut self, data: &[T]) {
        let size = std::mem::size_of_val(data);

        self.len = data.len();

        self.bind();

        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size as gl::types::GLsizeiptr,
                data.as_ptr() as *const _,
                self.ty.into(),
            );
        };

        Vbo::<T>::unbind();
    }

    pub fn setup(&self) -> &Vbo<T> {
        self.bind();
        let bindings = T::bindings();

        let stride = std::mem::size_of::<T>();

        // Init atrib locations
        for binding in bindings {
            let loc = binding.location as gl::types::GLuint;
            let ty = binding.ty;
            let elements = ty.get_num_components();
            let gl_ty = ty.get_gl_primative();

            if ty.is_integer() {
                unsafe {
                    gl::VertexAttribIPointer(
                        loc,
                        elements as gl::types::GLsizei,
                        gl_ty,
                        stride as gl::types::GLsizei,
                        binding.offset as *const () as *const _,
                    );
                }
            } else {
                unsafe {
                    gl::VertexAttribPointer(
                        loc,
                        elements as gl::types::GLsizei,
                        gl_ty,
                        gl::FALSE,
                        stride as gl::types::GLsizei,
                        binding.offset as *const () as *const _,
                    );
                }
            }
        }

        bindings.iter().for_each(|b| unsafe {
            gl::EnableVertexAttribArray(b.location as gl::types::GLuint);
        });

        if self.instance {
            bindings.iter().for_each(|b| unsafe {
                gl::VertexAttribDivisor(b.location as gl::types::GLuint, 1);
            });
        }
        Vbo::<T>::unbind();

        self
    }

    pub fn bind(&self) {
        unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, self.id) };
    }

    pub fn unbind() {
        unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, 0) };
    }
}

impl<T> Drop for Vbo<T>
where
    T: Vertex,
{
    fn drop(&mut self) {
        unsafe { gl::DeleteBuffers(1, &self.id) };
    }
}

pub enum Indices {
    None,
    Ebo(Ebo),
}

impl Indices {
    pub fn bind(&self) {
        match self {
            Indices::None => {}
            Indices::Ebo(ebo) => ebo.bind(),
        }
    }

    pub fn unbind(&self) {
        match self {
            Indices::None => {}
            Indices::Ebo(_) => Ebo::unbind(),
        }
    }
}

#[allow(dead_code)]
pub struct Ebo {
    id: gl::types::GLuint,
    len: usize,
    size: usize,
}

impl Ebo {
    pub fn new(data: &[u32], ty: DrawType) -> Self {
        let mut ebo = 0;

        let size = std::mem::size_of_val(data);

        unsafe {
            gl::GenBuffers(1, &mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                size as gl::types::GLsizeiptr,
                data.as_ptr() as *const _,
                ty.into(),
            );
        };

        Self {
            id: ebo,
            len: data.len(),
            size,
        }
    }

    pub fn bind(&self) {
        unsafe { gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.id) };
    }

    pub fn unbind() {
        unsafe { gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0) };
    }
}

impl Drop for Ebo {
    fn drop(&mut self) {
        unsafe { gl::DeleteBuffers(1, &self.id) };
    }
}
