
extern crate rustc_serialize;

extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use piston::input::Button::{ Keyboard, Mouse };
use piston::input::Input::{ Press, Release, Move };
use piston::input::Motion::{ MouseCursor, MouseScroll };
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };
use opengl_graphics::glyph_cache::GlyphCache;
use graphics::*;


mod world;
use world::*;


fn main() {
    // Change this to OpenGL::V2_1 if not working.
    //let opengl = OpenGL::V3_2;
    let opengl = OpenGL::V2_1;

    let mut window: Window = WindowSettings::new(
            "a bugs life",
            [1280, 720]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut gl = GlGraphics::new(opengl);
    let mut glyph = GlyphCache::new("assets/fonts/NotoSans/NotoSans-Regular.ttf").expect("Failed to load font");
    let mut app = App::new();

    let mut mouse_hold = false;
    let mut mouse_pos : [f64; 2] = [0.0, 0.0];

    let mut events = window.events().ups(120).max_fps(10_000);
    while let Some(e) = events.next(&mut window) {
	match e {
            Event::Render(_) => {
                if let Some(r) = e.render_args() {
                    gl.draw(r.viewport(), |c, gl| {
                        //clear([1.0; 4], gl);
                        app.render(&c, gl, &mut glyph, &r);
                    });
                    //app.render(&r);
                }
            },
            Event::Update(_) => {
                if let Some(u) = e.update_args() {
                    app.update(&u);
                }
            },
            Event::Input(Press(Keyboard(Key::Up))) => {
                app.viewport.origin[1] -= 1.0;
                if app.viewport.origin[1] < 0.0 {
                    app.viewport.origin[1] = 0.0;
                }
            },
            Event::Input(Press(Keyboard(Key::Down))) => {
                app.viewport.origin[1] += 1.0;
                if app.viewport.origin[1] >= app.world.terrain.size[1] as f64 {
                    app.viewport.origin[1] = app.world.terrain.size[1] as f64;
                }
            },
            Event::Input(Press(Keyboard(Key::Left))) => {
                app.viewport.origin[0] -= 1.0;
                if app.viewport.origin[0] < 0.0 {
                    app.viewport.origin[0] = 0.0;
                }
            },
            Event::Input(Press(Keyboard(Key::Right))) => {
                app.viewport.origin[0] += 1.0;
                if app.viewport.origin[0] >= app.world.terrain.size[0] as f64 {
                    app.viewport.origin[0] = app.world.terrain.size[0] as f64;
                }
            },
            Event::Input(Press(Keyboard(Key::NumPadPlus))) => {
                app.viewport.zoom += 1.0;
                if app.viewport.zoom >= 64.0 {
                    app.viewport.zoom = 64.0;
                }
            },
            Event::Input(Press(Keyboard(Key::NumPadMinus))) => {
                app.viewport.zoom -= 1.0;
                if app.viewport.zoom <= 4.0 {
                    app.viewport.zoom = 4.0;
                }
            },

            Event::Input(Press(Mouse(MouseButton::Left))) => {
                //println!("clicke");
            },
            Event::Input(Release(Mouse(MouseButton::Left))) => {
                app.find_closest(mouse_pos);
            },
            Event::Input(Move(MouseCursor(x, y))) => {
                mouse_pos = [ x, y ];
            },
            Event::Input(Move(MouseScroll(_, y))) => {
                app.viewport.zoom -= y as f64 * -1.0;
                if app.viewport.zoom <= 4.0 {
                    app.viewport.zoom = 4.0;
                } else if app.viewport.zoom >= 64.0 {
                    app.viewport.zoom = 64.0;
                }
            },
            Event::Input(Press(Keyboard(Key::O))) => {
                app.viewport.origin[0] = 0.0;
                app.viewport.origin[1] = 0.0;
            },

            Event::Input(Press(Keyboard(Key::C))) => {
		let mut n : Vec<CreatureID> = vec!(0, 0, 0, 0, 0, 0, 0);
		for creature in &app.world.creatures {
	            n[creature.ancestor as usize] += 1;
		}
                for i in n {
                    print!("{} ", i);
                }
                println!("");
	    },

            Event::Input(Press(Keyboard(Key::P))) => {
                app.world.run = !app.world.run;
	    },
            Event::Input(Press(Keyboard(Key::S))) => {
                if app.viewport.selected > 0 {
                    for creature in &app.world.creatures {
                        if creature.id == app.viewport.selected {
                            match creature.write(&format!("creatures/{}.json", creature.id)) {
                                Ok(_) => println!("Saved creature {}", creature.id),
                                Err(err) => println!("Error while saving creature {}: {}", creature.id, err),
                            }
                        }
                    }
                }
            },
            Event::Input(Press(Keyboard(Key::D))) => {
                if app.world.creatures.len() > 0 {
                    let encoded = rustc_serialize::json::encode(&app.world.creatures[0].brain).unwrap();
                    //let encoded = rustc_serialize::json::as_pretty_json(&c.brain);
                    println!("{}", encoded);
                }
            },
            Event::Input(Press(Keyboard(Key::I))) => {
                app.input_on = true;
                app.input_current = 0;
            },

            Event::Input(Press(Keyboard(Key::Return))) => {
                if app.input_on {
                    app.viewport.selected = app.input_current;
                    app.input_on = false;
                }
            },
            Event::Input(Press(Keyboard(key))) => {
                if app.input_on {
                    if key >= Key::D0 && key <= Key::D9 {
                        app.input_current = app.input_current * 10 + (key.code() - 0x30);
                    }
                }
            },

            _ => { },
	}
    }
}


const BORDER_WIDTH : u32 = 20;
const SIDE_WIDTH : u32 = 250;
const FONTSIZE : u32 = 20;


type ScreenPoint = [u32; 2];
type ScreenSize = [u32; 2];

struct WorldViewport {
    offset: ScreenPoint,
    size: ScreenSize,
    origin: WorldPoint,
    zoom: f64,
    selected: i32,
}

pub struct App {
    world: World,
    viewport: WorldViewport,

    input_on: bool,
    input_current: i32,
}

impl App {
    fn new() -> App
    {
        let viewport = WorldViewport {
            offset: [ BORDER_WIDTH, BORDER_WIDTH ],
            //size: [ args.width - SIDE_WIDTH - BORDER_WIDTH, args.height - (BORDER_WIDTH * 2) ],
            size: [ 1280, 720 ],
            origin: [ 0.0, 0.0 ],
            zoom: 7.0,
            selected: 0,
        };

        return App {
            world: World::new(),
            viewport: viewport,

            input_on: false,
            input_current: 0,
        };
    }

    fn render(&mut self, c: &Context, gl: &mut GlGraphics, glyph: &mut GlyphCache, args: &RenderArgs)
    {
        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        clear(BLACK, gl);

        self.viewport.size = [ args.width - SIDE_WIDTH - BORDER_WIDTH, args.height - (BORDER_WIDTH * 2) ];

        self.world.render(c, gl, glyph, &self.viewport);

        if self.input_on {
            let transform = c.transform.trans(self.viewport.offset[0] as f64, (self.viewport.size[1] + self.viewport.offset[1] + FONTSIZE) as f64);
            Text::new_color([1.0, 1.0, 1.0, 1.0], FONTSIZE).draw(&format!("> {}", self.input_current), glyph, &c.draw_state, transform, gl);        
        }
    }

    fn update(&mut self, args: &UpdateArgs)
    {
        // Rotate 2 radians per second.
        //self.rotation += 2.0 * args.dt

        self.world.timeslice();

        if self.world.time % 1000 == 0 && self.world.creatures.len() > 0 {
            let mut most_spawns = &self.world.creatures[0];
            let mut most_eaten = &self.world.creatures[0];
            for creature in &self.world.creatures {
                if creature.spawns > most_spawns.spawns {
                    most_spawns = creature;
                }

                if creature.eaten / (self.world.time - creature.birthday) as f64 > most_eaten.eaten / (self.world.time - most_eaten.birthday) as f64 {
                    most_eaten = creature;
                }
            }

            println!("\nOldest");
            self.world.creatures[0].print_info(self.world.time);

            println!("\nMost Spawns");
            most_spawns.print_info(self.world.time);

            println!("\nMost Eaten");
            most_eaten.print_info(self.world.time);
        }
    }

    fn find_closest(&mut self, mouse_pos: [f64; 2])
    {
        if self.world.creatures.len() <= 0 {
            return;
        }

        for creature in &self.world.creatures {
            let screen_pos = self.viewport.to_screen(creature.position);
            match screen_pos {
                None => {},
                Some(screen_pos) => {
                    if (screen_pos[0] as f64 - mouse_pos[0]).abs() < 10.0 && (screen_pos[1] as f64 - mouse_pos[1]).abs() < 10.0 {
                        self.viewport.selected = creature.id
                    }
                },
            }
        }
    }
}


impl WorldViewport {
    fn to_screen(&self, position : [f64; 2]) -> Option<[u32; 2]>
    {
        let screen = [
            self.offset[0].saturating_add(((position[0] - self.origin[0]) * self.zoom) as u32),
            self.offset[1].saturating_add(((position[1] - self.origin[1]) * self.zoom) as u32),
        ];

        if screen[0] < self.offset[0] || screen[0] > self.offset[0] + self.size[0] || screen[1] < self.offset[1] || screen[1] > self.offset[1] + self.size[1] {
            return None;
        } else {
            return Some(screen);
        }
    }
}


impl World {
    fn render(&self, c: &Context, gl: &mut GlGraphics, glyph: &mut GlyphCache, viewport: &WorldViewport)
    {
        self.terrain.render(c, gl, viewport);

        for creature in &self.creatures {
            creature.render(c, gl, glyph, viewport, self.time);
        }

        let lines = [
            &format!("Pop: {}", self.creatures.len()),
            &format!("Total: {}", self.total_lives),
            &format!("Food: {:.0}", self.terrain.total_food()),
            &format!("Season: {:.4}", self.terrain.season),
            &format!("Oldest: {} / {}", self.get_oldest(), self.time),
        ];

        for i in 0..lines.len() {
            let transform = c.transform.trans((viewport.size[0] + 20) as f64, (viewport.offset[1] + 20 + FONTSIZE * i as u32) as f64);
            Text::new_color([1.0, 1.0, 1.0, 1.0], FONTSIZE).draw(lines[i], glyph, &c.draw_state, transform, gl);        
        }
    }

    fn get_oldest(&self) -> u64
    {
        if self.creatures.len() <= 0 {
            return 0;
        }

        return self.time - self.creatures[0].birthday;
    }
}


impl Terrain {
    fn render(&self, c: &Context, gl: &mut GlGraphics, viewport: &WorldViewport)
    {
        let (mut x, mut y) = (viewport.offset[0], viewport.offset[1]);
        for col in viewport.origin[0] as usize..self.size[0] {
            for row in viewport.origin[1] as usize..self.size[1] {
                self.tiles[col][row].render(c, gl, x, y, viewport.zoom);
                y += viewport.zoom as u32;
                if y >= viewport.offset[1] + viewport.size[1] {
                    break;
                }
            }
            y = viewport.offset[0];
            x += viewport.zoom as u32;
            if x >= viewport.offset[0] + viewport.size[0] {
                break;
            }
        }


        rectangle([1.0, 0.0, 0.0, 1.0], [ 0.0, 0.0, 20.0, 1.0 ], c.transform.trans(viewport.offset[0] as f64, self.season_height as f64 * viewport.zoom + viewport.offset[1] as f64), gl);
    }
}


impl Tile {
    fn render(&self, c: &Context, gl: &mut GlGraphics, x : u32, y : u32, size : f64)
    {
        const BLACK : [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        let transform = c.transform.trans(x as f64, y as f64);
        //rectangle(self.colour(), rectangle::square(0.0, 0.0, 20.0), transform, gl);
        Rectangle::new(self.colour()).border(rectangle::Border { color: BLACK, radius: 1.0 }).draw(rectangle::square(0.0, 0.0, size), &c.draw_state, transform, gl);
    }

    fn colour(&self) -> [f32; 4]
    {
        //return graphics::math::hsv([1.0, 1.0, 1.0, 1.0], self.ttype as f32 / 5.0, 0.75, self.food as f32 / 100.0);

        //[ self.food as f32 / 100.0, self.ttype as f32 / 5.0, 0.75, 1.0 ]                          // red food, green type
        [ self.food as f32 / 100.0, self.food as f32 / 100.0, self.food as f32 / 100.0, 1.0 ]     // black and white food
    }
}


impl Creature {
    fn render(&self, c: &Context, gl: &mut GlGraphics, glyph: &mut GlyphCache, viewport: &WorldViewport, time: WorldTime)
    {
        const BLACK : [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        let colour : [f32; 4] = [self.birthday as f32 / time as f32, self.colour, 0.0, 1.0];

        let screen = match viewport.to_screen(self.position) {
            Some(s) => s,
            None => return,
        };
        let transform = c.transform.trans(screen[0] as f64, screen[1] as f64).rot_rad(self.angle);
        let size = self.size * viewport.zoom;

        if self.id == viewport.selected {
            ellipse([1.0, 0.0, 0.0, 1.0], rectangle::centered_square(0.0, 0.0, size / 2.0 + 2.0), transform, gl);
            self.render_info(c, gl, glyph, viewport, time);
        }

        ellipse(colour, rectangle::centered_square(0.0, 0.0, size / 2.0), transform, gl);
        rectangle(BLACK, [ 0.0, -1.0, size, 2.0 ], transform, gl);
    }

    fn render_info(&self, c: &Context, gl: &mut GlGraphics, glyph: &mut GlyphCache, viewport: &WorldViewport, time: WorldTime)
    {
        let lines = [
            &format!("ID: {}", self.id),
            &format!("Parent: {}", self.parent),
            &format!("Ancestor: {}", self.ancestor),
            &format!("Age: {:.4}", time - self.birthday),
            &format!("Spawns: {}", self.spawns),
            &format!("Eaten: {:.2}", self.eaten),
            &format!("Eaten/Y: {:.2}", self.eaten / (time - self.birthday) as f64),
        ];

        for i in 0..lines.len() {
            let transform = c.transform.trans((viewport.size[0] + 20) as f64, (viewport.offset[1] + 250 + FONTSIZE * i as u32) as f64);
            Text::new_color([1.0, 1.0, 1.0, 1.0], FONTSIZE).draw(lines[i], glyph, &c.draw_state, transform, gl);        
        }
    }

    fn print_info(&self, time: WorldTime)
    {
        let encoded = rustc_serialize::json::encode(&self.brain).unwrap();
        //let encoded = rustc_serialize::json::as_pretty_json(&self.world.creatures[0].brain);
        println!("id: {}", self.id);
        println!("ancestor: {}", self.ancestor);
        println!("age: {}", time - self.birthday);
        println!("colour: {}", self.colour);
        println!("size: {}", self.size);
        println!("spawns: {}", self.spawns);
        println!("eaten: {}", self.eaten / (time - self.birthday) as f64);
        println!("{}", encoded);
    }
}


