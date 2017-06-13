extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use std::time::{Instant};

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };

#[derive(Debug)]
struct Bullet {
    bullet: graphics::types::Rectangle,
    vector_x: f64,
    vector_y: f64,
    offset_x: f64,
    offset_y: f64,
    collision: bool,
}

struct Planet {
    render: graphics::types::Rectangle,
    x: f64,
    y: f64,
    size: f64,
    strength: f64,
}

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    projectiles: Vec<Bullet>,
    planets: [Planet; 3],
    i: u64,
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        const GREY:  [f32; 4] = [0.5, 0.5, 0.5, 1.0];
        const BLUE:  [f32; 4] = [0.0, 0.0, 1.0, 1.0];

        let ref mut planets = self.planets;
        let ref mut projectiles = self.projectiles;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            for p in projectiles {
                let b_transform = c.transform.trans(p.offset_x, p.offset_y);
                ellipse(BLUE, p.bullet, b_transform, gl);
            }

            for p in planets {
              let transform = c.transform.trans(p.x, p.y);
              let RG = 0.5 - (p.strength / 50.0);
              let color: [f32; 4] = [ RG as f32, RG as f32, (p.strength / 100.0) as f32, 1.0];
              ellipse(color, p.render, transform, gl);
            }
        });

    }

    fn update(&mut self, args: &UpdateArgs) {
        let g = 1.0;
        let m1 = 2.0;
        let m2 = 3.0;

        for pl in self.planets.iter() {
            for pr in &mut self.projectiles {
                // @TODO Verlet integration.
                // @TODO Fix proper angle.
                // Leaving a ^2 from gravity and a .sqrt from
                // Pythagoras as they cancel eachother out.
                let distance = (pl.x - pr.offset_x).powi(2) +
                               (pl.y - pr.offset_y).powi(2);
                let force = g * (m1 * m2) / distance;

                if pl.x < pr.offset_x {
                    pr.vector_x -= force;
                }
                else {
                    pr.vector_x += force;
                }
                if pl.y < pr.offset_y {
                    pr.vector_y -= force;
                }
                else {
                    pr.vector_y += force;
                }
            }
        }

        for pl in &mut self.planets {
            if pl.strength < 100.0 {
                pl.strength += 1.0 * args.dt;
            }
        }

        // Move projectiles around
        for pr in &mut self.projectiles {
          pr.offset_x += pr.vector_x;
          pr.offset_y += pr.vector_y;
        }

        // Collision?
        for pl in self.planets.iter() {
            for pr in &mut self.projectiles {
                let distance = ((pl.x - pr.offset_x).powi(2) +
                                (pl.y - pr.offset_y).powi(2)).sqrt();
                if (distance < pl.size / 10.0) {
                    pr.collision = true;
                }
            }
        }

        self.projectiles.retain(|p| p.collision == false);

        self.i += 1;
        if self.i % 500 == 0 {
            println!("{:?}", self.projectiles);
            //println!("{:?}", force);
        }
    }
}

fn main() {
    let mut now = Instant::now();
    let opengl = OpenGL::V3_2;
    let mut cursor = [0.0, 0.0];
    let mut cursor_x = 0.0;
    let mut cursor_y = 0.0;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new(
            "YASG!",
            [1024, 768]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        planets: [
            Planet {
                render:  graphics::rectangle::square(0.0, 0.0, 50.0),
                x: 300.0,
                y: 300.0,
                size: 50.0,
                strength: 1.0,
            },
            Planet {
                render:  graphics::rectangle::square(0.0, 0.0, 50.0),
                x: 500.0,
                y: 300.0,
                size: 50.0,
                strength: 10.0,
            },
            Planet {
                render:  graphics::rectangle::square(0.0, 0.0, 50.0),
                x: 700.0,
                y: 300.0,
                size: 50.0,
                strength: 1.0,
            } ],
        i: 0,
        projectiles: vec![],
    };

    let start_projectiles = Bullet {
        bullet: graphics::rectangle::square(0.0, 0.0, 10.0),
        vector_x: 0.0,
        vector_y: 0.3,
        offset_x: 120.0,
        offset_y: 300.0,
        collision: false,
    };
    app.projectiles.push(start_projectiles);
    app.projectiles.push(Bullet {
        bullet: graphics::rectangle::square(0.0, 0.0, 10.0),
        vector_x: 0.0,
        vector_y: -0.5,
        offset_x: 400.0,
        offset_y: 300.0,
        collision: false,
    });

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        let new_now = Instant::now().duration_since(now);
        now = Instant::now();
        let elapsed = (new_now.as_secs() * 1000000000) + new_now.subsec_nanos() as u64;

        if let Some(Button::Mouse(button)) = e.press_args() {
            println!("Pressed mouse button '{:?}'", button);
            println!("Mouse at '{} {}'", cursor_x, cursor_y);
            // println!("Gravity at '{} {}'", cursor_x, cursor_y);
            let g = 1.0;
            let displacement = (app.planets[0].x - cursor_x + app.planets[0].size / 2.0,
                                app.planets[0].y - cursor_y + app.planets[0].size / 2.0);
            let angle = displacement.1.atan2(displacement.0);
            let distance = displacement.0.powi(2) + displacement.1.powi(2);

            let force = 0.00003;//(g * 2.0 / distance).min(0.00005);
            let acceleration = (angle.cos() * force * 2.0e4, angle.sin() * force * 2.0e4);
            let start_offset = (35.0 * angle.cos(), 35.0 * angle.sin());

            app.projectiles.push(Bullet {
                bullet: graphics::rectangle::square(0.0, 0.0, 10.0),
                vector_x: acceleration.0,
                vector_y: acceleration.1,
                offset_x: app.planets[0].x + start_offset.0 + 25.0,
                offset_y: app.planets[0].y + start_offset.1 + 25.0,
                collision: false,
            });


        }
        if let Some(r) = e.render_args() {
            app.render(&r);
        }
        e.mouse_cursor(|x, y| {
            cursor = [x, y];
            cursor_x = x;
            cursor_y = y;
        });

        if let Some(u) = e.update_args() {
            // println!("{:?}", u);
            app.update(&u);
        }
    }
}

