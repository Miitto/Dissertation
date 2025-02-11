use std::{any::TypeId, cell::RefCell, collections::HashMap, rc::Rc};

use glium::glutin::surface::WindowSurface;
pub use shader_macros::program;
pub use shaders_common::ProgramInternal;

thread_local! {
static PROGRAMS: RefCell<HashMap<TypeId, Rc<glium::Program>>> =
    RefCell::new(HashMap::new());
}

thread_local! {
static PROGRAMS_COMPILED: RefCell<usize> = const { RefCell::new(0) };
}

pub trait Program {
    fn get(
        display: &glium::Display<WindowSurface>,
    ) -> Result<Rc<glium::Program>, glium::ProgramCreationError>;
}

impl<T: 'static + ProgramInternal> Program for T {
    fn get(
        display: &glium::Display<WindowSurface>,
    ) -> Result<Rc<glium::Program>, glium::ProgramCreationError> {
        // optick::event!("Program Get");
        let existing = PROGRAMS.with_borrow(|list| list.get(&TypeId::of::<T>()).cloned());

        if let Some(program) = existing {
            Ok(program)
        } else {
            let program = Rc::new(Self::to_glium(display)?);
            PROGRAMS_COMPILED.with_borrow_mut(|count| *count += 1);
            PROGRAMS.with_borrow_mut(|list| {
                list.insert(TypeId::of::<T>(), program.clone());
            });
            Ok(program)
        }
    }
}

pub fn shaders_compiled() -> usize {
    PROGRAMS_COMPILED.with_borrow(|count| *count)
}
