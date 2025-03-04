use std::{cell::RefCell, rc::Rc};

use crate::{
    DrawMode, DrawType,
    bounds::{BoundingHeirarchy, BoundingVolume as _},
    buffers::{EmptyVertex, Vao},
    vertex::Vertex,
};

pub struct Mesh<T, I = EmptyVertex>
where
    T: Vertex,
    I: Vertex,
{
    vertices: Vec<T>,
    indices: Option<Vec<u32>>,
    instances: Option<Rc<Vec<I>>>,
    bounds: BoundingHeirarchy,
    vao: RefCell<Option<Vao<T, I>>>,
    frustum_cull: bool,
    pub draw_mode: DrawMode,
    pub draw_type: DrawType,
    dirty_instance: RefCell<bool>,
}

impl<T> Mesh<T>
where
    T: Vertex,
{
    pub fn new(
        vertices: Vec<T>,
        indices: Option<Vec<u32>>,
        bounds: BoundingHeirarchy,
        draw_mode: DrawMode,
        draw_type: DrawType,
    ) -> Self {
        Self {
            vertices,
            indices,
            instances: None,
            bounds,
            frustum_cull: false,
            vao: RefCell::new(None),
            draw_mode,
            draw_type,
            dirty_instance: RefCell::new(false),
        }
    }
}

impl<T, I> Mesh<T, I>
where
    T: Vertex,
    I: Vertex,
{
    pub fn new_instance(
        vertices: Vec<T>,
        indices: Option<Vec<u32>>,
        instances: Vec<I>,
        bounds: BoundingHeirarchy,
        draw_mode: DrawMode,
        draw_type: DrawType,
    ) -> Self {
        Self {
            vertices,
            indices,
            instances: Some(Rc::new(instances)),
            bounds,
            frustum_cull: false,
            vao: RefCell::new(None),
            draw_mode,
            draw_type,
            dirty_instance: RefCell::new(false),
        }
    }

    pub fn instance_count(&self) -> i32 {
        self.instances.as_ref().unwrap().len() as i32
    }

    pub fn set_instances(&mut self, instances: Vec<I>) {
        self.set_instances_shared(Rc::new(instances));
    }

    pub fn set_instances_shared(&mut self, instances: Rc<Vec<I>>) {
        self.instances = Some(instances);

        *self.dirty_instance.borrow_mut() = true;
    }

    pub fn make_vao(&self) {
        let vao = Vao::<T, I>::new_maybe_instanced(
            &self.vertices,
            self.indices.as_deref(),
            self.draw_type,
            self.draw_mode,
            self.instances.as_ref().map(|i| i.as_slice()),
        );
        *self.vao.borrow_mut() = Some(vao);

        *self.dirty_instance.borrow_mut() = false;
    }

    pub fn set_bounds(&mut self, bounds: BoundingHeirarchy) {
        self.bounds = bounds;
    }

    pub fn frustum_cull(&self) -> bool {
        self.frustum_cull
    }

    pub fn set_frustum_cull(&mut self, frustum_cull: bool) {
        self.frustum_cull = frustum_cull;
    }

    pub fn is_on_frustum(&self, frustum: &crate::camera::frustum::Frustum) -> bool {
        self.bounds.intersects(frustum).into()
    }

    pub fn instanced(&self) -> bool {
        self.instances.is_some()
    }

    pub fn bind(&self) {
        self.ensure_vao();
        self.vao.borrow().as_ref().unwrap().bind();
    }

    pub fn ensure_vao(&self) {
        if self.vao.borrow().is_none() {
            self.make_vao();
        }

        if *self.dirty_instance.borrow() {
            self.vao
                .borrow_mut()
                .as_mut()
                .unwrap()
                .update_instances(self.instances.as_ref().unwrap());

            *self.dirty_instance.borrow_mut() = false;
        }
    }

    pub fn has_indices(&self) -> bool {
        self.indices.is_some()
    }

    pub fn len(&self) -> usize {
        if let Some(indices) = &self.indices {
            indices.len()
        } else {
            self.vertices.len()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
