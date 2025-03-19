#[derive(Debug, Clone)]
pub struct DrawArraysIndirectCommand {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first: u32,
    pub base_instance: u32,
}
