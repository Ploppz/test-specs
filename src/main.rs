extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate specs;
extern crate rand;

mod vec;
use vec::Vec2;

use rand::Rng;

use specs::{Gate, Join};

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };


#[derive(Clone, Debug)]
pub struct Pos (Vec2);
impl specs::Component for Pos {
    type Storage = specs::VecStorage<Pos>;
}


#[derive(Clone, Debug)]
pub struct Vel (Vec2);
impl specs::Component for Vel {
    type Storage = specs::VecStorage<Vel>;
}


#[derive(Clone, Debug)]
pub struct Force (Vec2);
impl specs::Component for Force {
    type Storage = specs::VecStorage<Force>;
}


#[derive(Clone, Debug)]
pub struct Attraction (Vec2);
impl specs::Component for Attraction {
    type Storage = specs::VecStorage<Attraction>;
}


#[derive(Clone, Debug)]
pub struct Ren;
impl specs::Component for Ren {
    type Storage = specs::VecStorage<Ren>;
}


//          \\
// System   \\
//          \\

struct AttractSys;
struct ReactSys;
struct MoveSys;
impl specs::System<()> for AttractSys {
    fn run(&mut self, arg: specs::RunArg, _: ()) {
        let (mut force, attraction, pos) = arg.fetch(|w| (w.write::<Force>(), w.read::<Attraction>(), w.read::<Pos>()) );
        for (force, attraction, pos) in (&mut force.pass(), &attraction.pass(), &pos.pass()).join() {
            force.0 += (pos.0 - attraction.0) * 0.1;
        }
    }
}

impl specs::System<()> for ReactSys {
    fn run(&mut self, arg: specs::RunArg, _: ()) {
        let (mut vel, force) = arg.fetch(|w| (w.write::<Vel>(), w.read::<Force>()) );
        for (vel, force) in (&mut vel.pass(), &force.pass()).join() {
            vel.0 += force.0;
        }
    }
}

impl specs::System<()> for MoveSys {
    fn run(&mut self, arg: specs::RunArg, _: ()) {
        let (mut pos, vel) = arg.fetch(|w| (w.write::<Pos>(), w.read::<Vel>()) );
        for (pos, vel) in (&mut pos.pass(), &vel.pass()).join() {
            pos.0 += vel.0;
        }
    }
}


pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    planner: specs::Planner<()>,
}

impl App {
    fn new(opengl: OpenGL) -> App {
        let mut rng = rand::thread_rng();

        let mut planner = {
            let mut w = specs::World::new();
            // All components types should be registered before working with them
            w.register::<Pos>();
            w.register::<Vel>();
            w.register::<Force>();
            w.register::<Attraction>();
            w.register::<Ren>();
            // create_now() of World provides with an EntityBuilder to add components to an Entity
            for _ in 0..10 {
                w.create_now()
                    .with(Pos (Vec2 {x: 0.0, y: 0.0}))
                    .with(Vel (Vec2 {x: rng.gen(), y: rng.gen()}))
                    .with(Force (Vec2 { x: 0.0, y: 0.0 }))
                    .with(Ren)
                    .build();
            }
            // Planner is used to run systems on the specified world with as many
            // threads as virtual cpus
            specs::Planner::<()>::new(w)
        };

        planner.add_system(MoveSys, "move system", 0);
        planner.add_system(ReactSys, "react system", 0);
        planner.add_system(AttractSys, "attract system", 0);

        App {
            gl: GlGraphics::new(opengl),
            planner: planner,
        }
    }
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;


        let w = self.planner.mut_world();
        let pos = w.read::<Pos>();
        let ren = w.read::<Ren>();

        let square = rectangle::square(0.0, 0.0, 50.0);
        let (x, y) = ((args.width / 2) as f64,
                      (args.height / 2) as f64);


        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear([0.5, 0.5, 0.5, 1.0], gl);
            for (pos, ren) in (&pos.pass(), &ren.pass()).join() {
                let transform = c.transform.trans(pos.0.x as f64, pos.0.y as f64);

                // Draw a box rotating around the middle of the screen.
                rectangle([1.0, 1.0, 1.0, 1.0], square, transform, gl);
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {

        self.planner.dispatch(());
        // Example of run_custom:
        /*
        self.planner.run_custom(|arg| {
            let (mut pos, vel) = arg.fetch(|w| {
                (w.write::<Pos>(), w.read::<Vel>())
            });

            for (pos, vel) in (&mut pos, &vel).join() {
                pos.offset(vel);
            }
        })
        */
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new(
            "spinning-square",
            [200, 200]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut app = App::new(opengl);


    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            app.render(&r);
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }
    }
}
