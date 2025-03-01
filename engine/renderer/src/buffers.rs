use std::{marker::PhantomData, rc::Rc};

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
    id: gl::types::GLuint,
    vbo: Rc<Vbo<T>>,
    instance_vbo: Option<Rc<Vbo<I>>>,
    indices: Rc<Indices>,
    ty: DrawType,
    pub mode: DrawMode,
    /// Store if we reused the vao when adding an instance vbo. Will stop us from deleting the
    /// vao twice
    moved_to_instance: bool,
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
            id: vao,
            ty,
            vbo: Rc::new(vbo),
            instance_vbo: None,
            indices: Rc::new(indices),
            mode,
            moved_to_instance: false,
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

        vao.moved_to_instance = true;

        Vao {
            id: vao.id,
            ty: vao.ty,
            vbo: vao.vbo.clone(),
            instance_vbo: Some(Rc::new(instance_vbo)),
            indices: vao.indices.clone(),
            mode: vao.mode,
            moved_to_instance: false,
        }
    }

    pub fn with_instance<O>(&self, instance: Rc<Vbo<O>>) -> Vao<T, O>
    where
        O: Vertex,
    {
        if !instance.instance {
            panic!("Attempted to use a none instance vbo as an instance vbo");
        }

        let mut vao = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
        }

        self.vbo.bind();
        self.vbo.setup();
        self.indices.bind();
        instance.bind();
        instance.setup();

        let vao = Vao {
            id: vao,
            ty: self.ty,
            vbo: self.vbo.clone(),
            instance_vbo: Some(instance),
            indices: self.indices.clone(),
            mode: self.mode,
            moved_to_instance: false,
        };

        vao.unbind_all();

        vao
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
        if let Indices::Ebo(ebo) = &*self.indices {
            ebo.len
        } else {
            self.vbo.len
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn has_indices(&self) -> bool {
        !matches!(&*self.indices, Indices::None)
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
        if self.moved_to_instance {
            return;
        }

        unsafe {
            gl::DeleteVertexArrays(1, &self.id);
        };
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
        };

        Vbo::<T>::unbind();

        vbo
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
