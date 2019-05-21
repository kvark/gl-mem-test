use gl;
use glutin;
use std::ptr;
use sysinfo::{SystemExt as _};

const TEXTURE_COUNT: u32 = 10;
const TEXTURE_SIZE: i32 = 2048;


#[derive(Debug)]
enum Allocation {
    Old { limit_max: bool },
    Storage,
}

#[derive(Debug)]
struct Test {
    target: gl::types::GLenum,
    alloc: Allocation,
}

const TESTS: &'static [Test] = &[
    Test {
        target: gl::TEXTURE_2D,
        alloc: Allocation::Old { limit_max: false },
    },
    Test {
        target: gl::TEXTURE_2D,
        alloc: Allocation::Old { limit_max: true },
    },
    Test {
        target: gl::TEXTURE_2D,
        alloc: Allocation::Storage,
    },
    Test {
        target: gl::TEXTURE_RECTANGLE,
        alloc: Allocation::Old { limit_max: false },
    },
    Test {
        target: gl::TEXTURE_RECTANGLE,
        alloc: Allocation::Storage,
    },
];

fn main() {
    println!("Starting gl-mem-test with {} textures of size {}x{}",
        TEXTURE_COUNT, TEXTURE_SIZE, TEXTURE_SIZE);
    println!("\tExpected: {} MB",
        TEXTURE_COUNT as usize * (TEXTURE_SIZE * TEXTURE_SIZE) as usize * 4 >> 20);

    let events_loop = glutin::EventsLoop::new();
    let mut system = sysinfo::System::new();
    let base = system.get_used_memory();
    let mut texture_ids = vec![0; TEXTURE_COUNT as usize];

    for test in TESTS {
        let builder = glutin::WindowBuilder::new();
        let context = glutin::ContextBuilder::new()
            .build_windowed(builder, &events_loop)
            .unwrap();

        let context = unsafe { context.make_current().unwrap() };
        gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

        unsafe {
           gl::GenTextures(TEXTURE_COUNT as _, texture_ids.as_mut_ptr());
        }
        for &tex in &texture_ids {
            unsafe {
                gl::BindTexture(test.target, tex);
                match test.alloc {
                    Allocation::Old { limit_max } => {
                        if limit_max {
                            gl::TexParameteri(test.target, gl::TEXTURE_MAX_LEVEL, 0);
                        }
                        gl::TexImage2D(test.target, 0, gl::RGBA8 as _, TEXTURE_SIZE, TEXTURE_SIZE, 0, gl::RGBA, gl::UNSIGNED_BYTE, ptr::null());
                    }
                    Allocation::Storage => {
                        gl::TexStorage2D(test.target, 1, gl::RGBA8 as _, TEXTURE_SIZE, TEXTURE_SIZE);
                    }
                }
            }
        }

        system.refresh_all();
        let kb = system.get_used_memory().max(base) - base;
        println!("\t{:?}: {} MB", test, kb >> 10);
        unsafe {
            gl::DeleteTextures(TEXTURE_COUNT as _, texture_ids.as_ptr());
            gl::Finish();
        }
        //context.swap_buffers().unwrap();
    }

    let err = unsafe { gl::GetError() };
    assert_eq!(0, err);
    println!("Done");
}
