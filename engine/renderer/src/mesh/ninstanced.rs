use crate::{
    DrawMode,
    bounds::{BoundingHeirarchy, BoundingVolume},
    buffers::{Buffer, BufferError, BufferMode, Ebo, FencedBuffer, Vao, Vbo},
    vertex::Vertex,
};

use super::Mesh;

pub struct NInstancedMesh<V, I>
where
    V: Vertex,
    I: Vertex,
{
    vbo: Vbo<V>,
    instance_buffers: Vec<Vbo<I>>,
    indices: Option<Ebo>,
    bounds: BoundingHeirarchy,
    vao: Vao<V, I>,
    setup_instances: bool,
    frustum_cull: bool,
}

impl<V, I> NInstancedMesh<V, I>
where
    V: Vertex,
    I: Vertex,
{
    pub fn with_vertices(
        verts: &[V],
        indices: Option<&[u32]>,
        draw_mode: DrawMode,
    ) -> Result<Self, BufferError> {
        let vbo = Vbo::with_data(verts, BufferMode::Immutable)?;

        let vao = Vao::new(draw_mode);
        vao.setup_vertices(&vbo);

        let indices = indices.map(|i| {
            Ebo::with_data(i, BufferMode::Immutable)
                .expect("Failed to create EBO for NInstancedMesh")
        });

        if let Some(indices) = &indices {
            vao.setup_indices(indices);
        }

        Ok(Self {
            vbo,
            instance_buffers: vec![],
            indices,
            bounds: BoundingHeirarchy::default(),
            vao,
            setup_instances: false,
            frustum_cull: false,
        })
    }

    pub fn enable_frustum_culling(&mut self) {
        self.frustum_cull = true;
    }

    fn get_first_available(&mut self) -> Option<usize> {
        self.instance_buffers.iter().position(|b| b.signalled())
    }

    pub fn set_instances(&mut self, data: &[I]) -> Result<(), BufferError> {
        let writable_idx = self.get_first_available();
        if let Some(writable_idx) = writable_idx {
            let writable = &mut self.instance_buffers[writable_idx];

            writable.set_data(data)?;

            let updated = self.instance_buffers.remove(writable_idx);
            self.instance_buffers.insert(0, updated);
        } else {
            let buffer = Vbo::with_data(data, BufferMode::Persistent)?;
            self.instance_buffers.insert(0, buffer);
        }

        let buf = &self.instance_buffers[0];
        if !self.setup_instances {
            println!("Setting up instances");
            self.vao.setup_instances(buf);
        }

        Ok(())
    }

    fn pre_draw(&mut self) -> (&Vao<V, I>, &mut Vbo<I>) {
        let idx = self.get_first_available();
        let last = self.instance_buffers.len() - 1;

        if idx.is_none() {
            println!("Using last");
        }

        let drawable = &mut self.instance_buffers[idx.unwrap_or(last)];
        if !self.setup_instances {
            self.vao.setup_instances(drawable);
        } else {
            self.vao.set_instance_vbo(drawable.id());
        }

        (&self.vao, drawable)
    }

    fn is_on_frustum(&self, frustum: &crate::camera::frustum::Frustum) -> bool {
        self.bounds.intersects(frustum).into()
    }
}

impl<V, I> Mesh<V, I> for NInstancedMesh<V, I>
where
    V: Vertex,
    I: Vertex,
{
    fn set_bounds(&mut self, bounds: crate::bounds::BoundingHeirarchy) {
        self.bounds = bounds;
    }

    fn bind(&self) {
        self.vao.bind();
    }

    fn draw_mode(&self) -> crate::DrawMode {
        self.vao.draw_mode()
    }

    fn vertex_count(&self) -> usize {
        if let Some(i) = &self.indices {
            i.count()
        } else {
            self.vbo.count()
        }
    }

    fn instance_count(&self) -> usize {
        todo!()
    }

    fn is_instanced(&self) -> bool {
        true
    }

    fn has_indices(&self) -> bool {
        self.indices.is_some()
    }

    fn render(&mut self, frustum: &crate::camera::frustum::Frustum) {
        if self.instance_buffers.is_empty() {
            return;
        }

        if self.frustum_cull && !self.is_on_frustum(frustum) {
            return;
        }

        let vertex_count = self.vertex_count() as i32;
        let mode = self.draw_mode().into();

        if vertex_count <= 0 {
            return;
        }

        let indices = self.has_indices();
        let (vao, buffer) = self.pre_draw();
        let instances = buffer.count() as i32;

        vao.bind();

        if instances <= 0 {
            return;
        }

        if indices {
            unsafe {
                gl::DrawElementsInstanced(
                    mode,
                    vertex_count,
                    gl::UNSIGNED_INT,
                    std::ptr::null(),
                    instances,
                );
            }
        } else {
            unsafe {
                gl::DrawArraysInstanced(mode, 0, vertex_count, instances);
            }
        }

        buffer.start_fence();
    }
}
