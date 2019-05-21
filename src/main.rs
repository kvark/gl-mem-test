use gl;
use glutin;
use std::{env, ptr, thread, time::Duration};
use sysinfo::{SystemExt as _};

const SPAWN_THREAD: bool = false;
const DELAY_SEC: u64 = 1;
const TEXTURE_COUNT: u32 = 100;
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

impl Test {
    fn run(&self) -> u64 {
        let events_loop = glutin::EventsLoop::new();
        let builder = glutin::WindowBuilder::new();
        let context = glutin::ContextBuilder::new()
            .build_windowed(builder, &events_loop)
            .unwrap();

        let context = unsafe { context.make_current().unwrap() };
        gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

        let mut texture_ids = vec![0; TEXTURE_COUNT as usize];
        unsafe {
           gl::GenTextures(TEXTURE_COUNT as _, texture_ids.as_mut_ptr());
        }

        for &tex in &texture_ids {
            unsafe {
                gl::BindTexture(self.target, tex);
                match self.alloc {
                    Allocation::Old { limit_max } => {
                        if limit_max {
                            gl::TexParameteri(self.target, gl::TEXTURE_MAX_LEVEL, 0);
                        }
                        gl::TexImage2D(self.target, 0, gl::RGBA8 as _, TEXTURE_SIZE, TEXTURE_SIZE, 0, gl::RGBA, gl::UNSIGNED_BYTE, ptr::null());
                    }
                    Allocation::Storage => {
                        gl::TexStorage2D(self.target, 1, gl::RGBA8 as _, TEXTURE_SIZE, TEXTURE_SIZE);
                    }
                }
            }
        }

        let system = sysinfo::System::new();
        unsafe {
            gl::DeleteTextures(TEXTURE_COUNT as _, texture_ids.as_ptr());
            gl::Finish();
            assert_eq!(0, gl::GetError());
        }
        //context.swap_buffers().unwrap();
        system.get_used_memory()
    }
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let base = sysinfo::System::new().get_used_memory();

    if args.len() > 1 {
        let test = Test {
            target: match args[1].as_str() {
                "2d" => gl::TEXTURE_2D,
                "rect" => gl::TEXTURE_RECTANGLE,
                other => panic!("Wrong target '{}'", other),
            },
            alloc: match args[2].as_str() {
                "storage" => Allocation::Storage,
                "image" => Allocation::Old {
                    limit_max: args.get(3).map_or(false, |a| a.parse().unwrap()),
                },
                other => panic!("Wrong alloc '{}'", other),
            },
        };
        let kb = test.run().max(base) - base;
        println!("{:?}: {} MB", test, kb >> 10);
    } else {
        println!("Starting gl-mem-test with {} textures of size {}x{}",
            TEXTURE_COUNT, TEXTURE_SIZE, TEXTURE_SIZE);
        println!("\tExpected: {} MB",
            TEXTURE_COUNT as usize * (TEXTURE_SIZE * TEXTURE_SIZE) as usize * 4 >> 20);

        for test in TESTS {
            let new = if SPAWN_THREAD {
                thread::spawn(move || test.run()).join().unwrap()
            } else {
                test.run()
            };

            let kb = new.max(base) - base;
            println!("\t{:?}: {} MB", test, kb >> 10);

            thread::sleep(Duration::from_millis(DELAY_SEC * 1000));
        }

        println!("Done");
    }
}
