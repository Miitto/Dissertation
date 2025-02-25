use std::{any::TypeId, cell::RefCell, collections::HashMap, rc::Rc};

use render_common::make_program;
pub use render_common::shaders::ProgramInternal;
pub use shader_macros::program;

thread_local! {
static PROGRAMS: RefCell<HashMap<TypeId, Rc<render_common::Program>>> =
    RefCell::new(HashMap::new());
}

thread_local! {
static PROGRAMS_COMPILED: RefCell<usize> = const { RefCell::new(0) };

}

pub trait Program {
    fn get() -> Rc<render_common::Program>;
}

impl<T: 'static + ProgramInternal> Program for T {
    fn get() -> Rc<render_common::Program> {
        // optick::event!("Program Get");
        let existing = PROGRAMS.with_borrow(|list| list.get(&TypeId::of::<T>()).cloned());

        if let Some(program) = existing {
            program
        } else {
            let program = Rc::new(make_program(Self::vertex(), Self::fragment()));
            PROGRAMS_COMPILED.with_borrow_mut(|count| *count += 1);
            PROGRAMS.with_borrow_mut(|list| {
                list.insert(TypeId::of::<T>(), program.clone());
            });
            program
        }
    }
}

pub fn shaders_compiled() -> usize {
    PROGRAMS_COMPILED.with_borrow(|count| *count)
}
