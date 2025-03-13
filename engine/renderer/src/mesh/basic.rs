use crate::{
    DrawMode,
    bounds::{BoundingHeirarchy, BoundingVolume},
    buffers::{Buffer, BufferMode, Ebo, EmptyVertex, Vao, Vbo},
    indices::Indices,
    vertex::Vertex,
};

use super::Mesh;

pub struct BasicMesh<V, I = EmptyVertex>
where
    V: Vertex,
    I: Vertex,
{
    vbo: Vbo<V>,
    indices: Option<Ebo>,
    instances: Option<Vbo<I>>,
    vao: Vao<V, I>,
    bounds: BoundingHeirarchy,
    setup_verts: bool,
    setup_instances: bool,
    frustum_cull: bool,
}

impl<V, I> BasicMesh<V, I>
where
    V: Vertex,
    I: Vertex,
{
    pub fn from_buffers(
        vertex: Vbo<V>,
        indices: Option<Ebo>,
        instances: Option<Vbo<I>>,
        bounds: Option<BoundingHeirarchy>,
        draw_mode: DrawMode,
    ) -> Self {
        let vao = Vao::new(draw_mode);
        vao.setup_vertices(&vertex);
        if let Some(indices) = &indices {
            vao.setup_indices(indices);
        }

        if let Some(instances) = &instances {
            vao.setup_instances(instances);
        }

        let setup_instances = instances.is_some();

        Self {
            vbo: vertex,
            indices,
            instances,
            vao,
            bounds: bounds.unwrap_or_default(),
            setup_verts: true,
            setup_instances,
            frustum_cull: false,
        }
    }

    pub fn from_data(
        vertex: &[V],
        indices: Option<&[u32]>,
        instances: Option<&[I]>,
        bounds: Option<BoundingHeirarchy>,
        mutable_vertices: bool,
        mutable_instances: bool,
        draw_mode: DrawMode,
    ) -> Self {
        let vbo = Vbo::with_data(
            vertex,
            if mutable_vertices {
                BufferMode::Persistent
            } else {
                BufferMode::Immutable
            },
        )
        .unwrap();
        let indices = indices.map(|i| {
            Ebo::with_data(
                i,
                if mutable_vertices {
                    BufferMode::Persistent
                } else {
                    BufferMode::Immutable
                },
            )
            .unwrap()
        });
        let instances = instances.map(|i| {
            Vbo::with_data(
                i,
                if mutable_instances {
                    BufferMode::Persistent
                } else {
                    BufferMode::Immutable
                },
            )
            .unwrap()
        });

        let vao = Vao::new(draw_mode);
        vao.setup_vertices(&vbo);
        if let Some(indices) = &indices {
            vao.setup_indices(indices);
        }

        if let Some(instances) = &instances {
            vao.setup_instances(instances);
        }

        let setup_instances = instances.is_some();

        Self {
            vbo,
            indices,
            instances,
            vao,
            bounds: bounds.unwrap_or_default(),
            setup_verts: true,
            setup_instances,
            frustum_cull: false,
        }
    }

    pub fn empty(size: usize, mutable_vertices: bool, draw_mode: DrawMode) -> Self {
        let vbo = Vbo::empty(
            size,
            if mutable_vertices {
                BufferMode::Persistent
            } else {
                BufferMode::Immutable
            },
        )
        .unwrap();

        Self {
            vbo,
            indices: None,
            instances: None,
            vao: Vao::new(draw_mode),
            bounds: BoundingHeirarchy::default(),
            setup_verts: false,
            setup_instances: false,
            frustum_cull: false,
        }
    }

    pub fn enable_frustum_cull(&mut self) {
        self.frustum_cull = true;
    }

    fn is_on_frustum(&self, frustum: &crate::camera::frustum::Frustum) -> bool {
        !self.frustum_cull || self.bounds.intersects(frustum).into()
    }

    pub fn set_vertices(&mut self, vertices: &[V]) -> Result<(), crate::buffers::BufferError> {
        let realloc = self.vbo.set_data(vertices)?;

        if !self.setup_verts {
            self.vao.setup_vertices(&self.vbo);
            self.setup_verts = true;
        } else if realloc {
            self.vao.set_vert_vbo(self.vbo.id());
        }

        Ok(())
    }

    pub fn set_indices(&mut self, indices: &[u32]) -> Result<(), crate::buffers::BufferError> {
        if let Some(ind) = &mut self.indices {
            if ind.set_data(indices)? {
                self.vao.setup_indices(ind);
            }
        } else {
            let indices = Ebo::with_data(indices, crate::buffers::BufferMode::Immutable)?;
            self.vao.setup_indices(&indices);
            self.indices.replace(indices);
        }

        Ok(())
    }

    pub fn set_instances(&mut self, instances: &[I]) -> Result<(), crate::buffers::BufferError> {
        if let Some(inst) = &mut self.instances {
            if inst.set_data(instances)? {
                self.vao.set_instance_vbo(inst.id());
            }
        } else {
            let instances = Vbo::with_data(instances, crate::buffers::BufferMode::Persistent)?;
            self.instances.replace(instances);
            self.vao.setup_instances(self.instances.as_ref().unwrap());
            self.setup_instances = true;
        }

        Ok(())
    }
}

impl<V, I> Mesh<V, I> for BasicMesh<V, I>
where
    V: Vertex,
    I: Vertex,
{
    fn set_bounds(&mut self, bounds: BoundingHeirarchy) {
        self.bounds = bounds;
    }

    fn bind(&self) {
        self.vao.bind();
    }

    fn draw_mode(&self) -> DrawMode {
        self.vao.draw_mode()
    }

    fn vertex_count(&self) -> usize {
        if let Some(indices) = &self.indices {
            Indices::count(indices)
        } else {
            self.vbo.count()
        }
    }

    fn instance_count(&self) -> usize {
        self.instances
            .as_ref()
            .map(|i| i.count())
            .unwrap_or_default()
    }

    fn is_instanced(&self) -> bool {
        self.instances.is_some()
    }

    fn has_indices(&self) -> bool {
        self.indices.is_some()
    }

    fn render(&mut self, frustum: &crate::camera::frustum::Frustum) {
        if self.frustum_cull && !self.is_on_frustum(frustum) {
            return;
        }

        self.bind();

        let vertex_count = self.vertex_count() as i32;
        let mode = self.draw_mode().into();

        if vertex_count <= 0 {
            return;
        }

        let indices = self.has_indices();
        let instanced = self.is_instanced();
        let instances = self.instance_count() as i32;

        if instanced && instances <= 0 {
            return;
        }

        match (indices, instanced) {
            (true, true) => unsafe {
                gl::DrawElementsInstanced(
                    mode,
                    vertex_count,
                    gl::UNSIGNED_INT,
                    std::ptr::null(),
                    instances,
                );
            },
            (false, true) => unsafe {
                gl::DrawArraysInstanced(mode, 0, vertex_count, instances);
            },
            (true, false) => unsafe {
                gl::DrawElements(mode, vertex_count, gl::UNSIGNED_INT, std::ptr::null());
            },
            (false, false) => unsafe {
                gl::DrawArrays(mode, 0, vertex_count);
            },
        }
    }
}
