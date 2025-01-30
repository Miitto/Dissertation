use glium::glutin::surface::WindowSurface;
pub use shader_macros::program;

pub trait Program {
    fn vertex() -> &'static str;
    fn fragment() -> &'static str;
    fn geometry() -> Option<&'static str>;
    fn to_glium(
        display: &glium::Display<WindowSurface>,
    ) -> Result<glium::Program, glium::ProgramCreationError> {
        glium::Program::from_source(display, Self::vertex(), Self::fragment(), Self::geometry())
    }
}
