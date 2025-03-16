use std::{any::TypeId, cell::RefCell, collections::HashMap, rc::Rc};

pub use render_common::shaders::ComputeProgram as ComputeProgramInternal;
pub use render_common::shaders::ProgramInternal;
use render_common::{make_compute_program, make_program};
pub use shader_macros::{compute, program, snippet};

thread_local! {
static PROGRAMS: RefCell<HashMap<TypeId, Rc<render_common::Program>>> =
    RefCell::new(HashMap::new());
static COMPUTE_PROGRAMS: RefCell<HashMap<TypeId, Rc<render_common::ComputeProgram>>> = RefCell::new(HashMap::new());
}

thread_local! {
static PROGRAMS_COMPILED: RefCell<usize> = const { RefCell::new(0) };

}

pub trait Program {
    fn get() -> Rc<render_common::Program>;
}

pub trait ComputeProgram {
    fn get() -> Rc<render_common::ComputeProgram>;
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

impl<T: 'static + ComputeProgramInternal> ComputeProgram for T {
    fn get() -> Rc<render_common::ComputeProgram> {
        // optick::event!("Program Get");
        let existing = COMPUTE_PROGRAMS.with_borrow(|list| list.get(&TypeId::of::<T>()).cloned());

        if let Some(program) = existing {
            program
        } else {
            let program = Rc::new(make_compute_program(Self::compute()));
            PROGRAMS_COMPILED.with_borrow_mut(|count| *count += 1);
            COMPUTE_PROGRAMS.with_borrow_mut(|list| {
                list.insert(TypeId::of::<T>(), program.clone());
            });
            program
        }
    }
}

pub fn shaders_compiled() -> usize {
    PROGRAMS_COMPILED.with_borrow(|count| *count)
}
