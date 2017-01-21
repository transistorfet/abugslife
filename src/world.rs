
use std::f64;
use std::fs::File;
use std::io::{ self, Write };

extern crate rand;
use self::rand::Rng;

extern crate rustc_serialize;
//use self::rustc_serialize::*;


pub type WorldTime = u64;

pub struct World {
    pub time: WorldTime,
    pub region: Region,
    pub creatures: Vec<Creature>,
    pub total_lives: usize,
}

impl World {
    pub fn new() -> World
    {
        let region = Region::new();

        let mut creatures : Vec<Creature> = Vec::new();
        for _ in 0..CREAT_INIT {
            let position = ( rand::thread_rng().gen_range(0.0, region.size[0] as f64), rand::thread_rng().gen_range(0.0, region.size[1] as f64) );
            let size = rand::thread_rng().gen_range(0.75, 1.25);
            let colour = rand::thread_rng().gen_range(0.0, 1.0);

            let creature = Creature::new(position.0, position.1, size, 0.05, 0.0, 1, colour, None);
            creatures.push(creature);
        }

        World {
            time: 1,
            region: region,
            total_lives: creatures.len(),
            creatures: creatures,
        }
    }

    pub fn timeslice(&mut self)
    {
        if self.creatures.len() <= 0 {
            return;
        }

        self.time += 1;

        self.region.timeslice(self.time);

        let mut newcreats : Vec<Creature> = vec!();

        for creature in &mut self.creatures {
            creature.timeslice(&mut self.region);

            //if self.time - creature.lastbirth > 1000 && creature.size > 0.75 {
            if self.time - creature.lastbirth > 100 && creature.size > 0.75 && rand::thread_rng().gen_range(0.0, 1.0) <= 0.001 {
                creature.lastbirth = self.time;
                let mut newcreature = creature.spawn(self.time);
                self.total_lives += 1;
                newcreature.position = self.region.wrap_position(newcreature.position);
                newcreats.push(newcreature);
            }
        }

        for newcreat in newcreats {
            self.creatures.push(newcreat);
        }

        self.creatures.retain(|ref creature| creature.size >= 0.25);
    }
}


const WORLD_WIDTH: usize = 200;
const WORLD_HEIGHT: usize = 100;

pub type WorldPoint = [f64; 2];

pub struct Region {
    pub size: [usize; 2],
    pub tiles: [[Tile; WORLD_HEIGHT]; WORLD_WIDTH],

    pub season: f64,
}

impl Region {
    fn new() -> Region
    {
        let mut tiles = [[Tile { ttype: 0, food: 0.0 }; WORLD_HEIGHT]; WORLD_WIDTH];
        for col in 0..WORLD_WIDTH {
            for row in 0..WORLD_HEIGHT {
                tiles[col][row] = Tile::new();
            }
        }

        Region {
            size: [ WORLD_WIDTH, WORLD_HEIGHT ],
            tiles: tiles,
            season: 0.0,
        }
    }

    fn wrap_position(&self, position : WorldPoint) -> WorldPoint
    {
        let mut newpos : WorldPoint = [0.0, 0.0];
        if position[0] < 0.0 { newpos[0] = self.size[0] as f64 - 0.1 } else if position[0] >= self.size[0] as f64 { newpos[0] = 0.0 } else { newpos[0] = position[0] }
        if position[1] < 0.0 { newpos[1] = self.size[1] as f64 - 0.1 } else if position[1] >= self.size[1] as f64 { newpos[1] = 0.0 } else { newpos[1] = position[1] }
        return newpos;
    }

    pub fn total_food(&self) -> f64
    {
        let mut sum : f64 = 0.0;

        for col in 0..self.size[0] {
            for row in 0..self.size[1] {
                sum += self.tiles[col][row].food;
            }
        }
        return sum;
    }

    fn timeslice(&mut self, time : WorldTime)
    {
        self.season = (time as f64 / 100.0).sin().max(0.0);

        //// Grow new food over time
        if time % 10 == 0 {
            for col in 0..self.size[0] {
                for row in 0..self.size[1] {
                    self.tiles[col][row].food += rand::thread_rng().gen_range(0.0, 1.0) * self.season;
                    if self.tiles[col][row].food > 100.0 {
                        self.tiles[col][row].food = 100.0;
                    }
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
pub struct Tile {
    pub ttype: i32,
    pub food: f64,
}

impl Tile {
    fn new() -> Tile
    {
        return Tile {
            ttype: rand::thread_rng().gen_range(0, 5),
            food: rand::thread_rng().gen_range(0.0, 100.0),
        };
    }

    fn feed(&mut self) -> f64
    {
        let r = self.food.min(rand::thread_rng().gen_range(0.1, 1.0));
        self.food -= r;
        return r;
    }
}



const CREAT_INIT : i32 = 40;

pub struct Creature {
    pub birthday: WorldTime,
    pub lastbirth: WorldTime,
    pub colour: f32,

    pub position: [f64; 2],
    pub size: f64,
    pub speed: f64,
    pub angle: f64,

    pub brain: Brain,
}

impl Creature {
    fn new(x : f64, y: f64, size: f64, speed: f64, angle: f64, birthday: WorldTime, colour: f32, brain: Option<Brain>) -> Creature
    {
        let newbrain = match brain {
            Some(brain) => brain,
            None => Brain::new()
        };

        Creature {
            birthday: birthday,
            lastbirth: birthday,
            colour: colour,
            position: [ x, y ],
            size: size,
            speed: speed + rand::thread_rng().gen_range(-0.2, 0.2),
            angle: angle + rand::thread_rng().gen_range(-0.4, 0.4),
            brain: newbrain,
        }
    }

    fn spawn(&mut self, birthday: WorldTime) -> Creature
    {
        let newcolour = self.colour + rand::thread_rng().gen_range(-0.1 as f32, 0.1 as f32).min(1.0).max(0.0);
        //let size = self.size + rand::thread_rng().gen_range(-0.25, 0.25);
        let size = self.size / 2.0;
        self.size -= size;
        return Creature::new(self.position[0] + 2.0, self.position[1] + 2.0, size, self.speed, self.angle, birthday, newcolour, Some(self.brain.spawn()));
    }

    fn timeslice(&mut self, region : &mut Region)
    {
        //self.x += rand::thread_rng().gen_range(-0.05, 0.05);
        //self.y += rand::thread_rng().gen_range(-0.05, 0.05);

        //self.angle += rand::thread_rng().gen_range(-0.1, 0.1);
        //self.speed += rand::thread_rng().gen_range(-0.001, 0.001);

        let foodbelow = region.tiles[self.position[0] as usize][self.position[1] as usize].food;
        let infront = region.wrap_position([ self.position[0] + 1.0 * self.angle.cos(), self.position[1] + 1.0 * self.angle.sin() ]);
        let foodahead = region.tiles[infront[0] as usize][infront[1] as usize].food;
        let leftfront = region.wrap_position([ self.position[0] + 1.0 * (self.angle + f64::consts::PI / 4.0).cos(), self.position[1] + 1.0 * (self.angle + f64::consts::PI / 4.0).sin() ]);
        let foodleft = region.tiles[leftfront[0] as usize][leftfront[1] as usize].food;
        let rightfront = region.wrap_position([ self.position[0] + 1.0 * (self.angle - f64::consts::PI / 4.0).cos(), self.position[1] + 1.0 * (self.angle - f64::consts::PI / 4.0).sin() ]);
        let foodright = region.tiles[rightfront[0] as usize][rightfront[1] as usize].food;

        let input : Vec<f64> = vec!(foodbelow, foodahead, foodleft, foodright, self.size, self.angle, self.speed);
        let output = match self.brain.forward(&input) {
            Some(output) => output,
            None => return,
        };

        //self.angle += (output[0] - 0.5) * 0.1;
        //self.speed += (output[1] - 0.5) * 0.001;
        self.angle += if output[0] > 0.5 { 0.2 } else if output[1] > 0.5 { -0.2 } else { 0.0 };
        self.speed = if output[2] > 0.5 { 0.2 } else { 0.001 };

        self.speed = self.speed.max(0.0).min(1.0);

        self.position[0] += self.speed * self.angle.cos();
        self.position[1] += self.speed * self.angle.sin();
        self.position = region.wrap_position(self.position);

        self.size -= self.size * 0.005;  // cost to live
        let food = region.tiles[self.position[0] as usize][self.position[1] as usize].feed();
        self.size += ((1.0 / self.size).powf(2.0) * food * 0.01) - 0.005;
    }

    pub fn write(&self) -> Result<(), io::Error>
    {
        let mut f = match File::create("creature.json") {
            Ok(f) => f,
            Err(err) => return Err(err),
        };

        let encoded = rustc_serialize::json::as_pretty_json(&self.brain);
        match f.write(format!("{}", encoded).as_bytes()) {
            Ok(_) => return Ok(()),
            Err(err) => return Err(err),
        };
    }
}


const BRAIN_IN : u32 = 7;
const BRAIN_L1 : u32 = 10;
const BRAIN_L2 : u32 = 10;
const BRAIN_OUT : u32 = 3;

#[derive(RustcDecodable, RustcEncodable)]
pub struct Brain {
    layers: Vec<Layer>,
}

impl Brain {
    fn new() -> Brain
    {
        let mut layers : Vec<Layer> = vec!();
        layers.push(Layer::new(BRAIN_IN, BRAIN_L1));
        layers.push(Layer::new(BRAIN_L1, BRAIN_L2));
        layers.push(Layer::new(BRAIN_L2, BRAIN_OUT));

        Brain {
            layers: layers,
        }
    }

    fn spawn(&self) -> Brain
    {
        let mut layers : Vec<Layer> = vec!();
        layers.push(self.layers[0].spawn());
        layers.push(self.layers[1].spawn());

        Brain {
            layers: layers,
        }
    }

    fn forward(&mut self, input : &Vec<f64>) -> Option<Vec<f64>>
    {
        // TODO
        // result = input;
        //for each layer in brain
        //  result = layer.forward(result);
        // return result;

        let mut output = input.to_vec();
        for layer in &self.layers {
            match layer.forward(&output) {
                Some(result) => output = result,
                None => {
                    println!("mismatched matrix multiplication");
                    return None;
                }
            }
        }
        return Some(output);
    }
}

#[allow(non_snake_case)]
#[derive(RustcDecodable, RustcEncodable)]
pub struct Layer {
    W: Vec<Vec<f64>>,
    b: Vec<f64>,
}

#[allow(non_snake_case)]
impl Layer {
    fn new(width: u32, height: u32) -> Layer
    {
        let mut W : Vec<Vec<f64>> = vec!();
        for _ in 0..height {
            let mut Wv : Vec<f64> = vec!();
            for _ in 0..width {
                Wv.push(rand::thread_rng().gen_range(-1.0, 1.0));
            }
            W.push(Wv);
        }

        let mut b : Vec<f64> = vec!();
        for _ in 0..height {
            b.push(rand::thread_rng().gen_range(-1.0, 1.0));
        }

        Layer {
            W: W,
            b: b,
        }
    }

    fn spawn(&self) -> Layer
    {
        let mut W : Vec<Vec<f64>> = vec!();
        for v in 0..self.W.len() {
            let mut Wv : Vec<f64> = vec!();
            for u in 0..self.W[v].len() {
                Wv.push((self.W[v][u] + rand::thread_rng().gen_range(-0.4 as f64, 0.4 as f64).powf(3.0)).min(1.0).max(-1.0));
            }
            W.push(Wv);
        }

        let mut b : Vec<f64> = vec!();
        for v in 0..self.b.len() {
            b.push((self.b[v] + rand::thread_rng().gen_range(-0.4 as f64, 0.4 as f64).powf(3.0)).min(1.0).max(-1.0));
        }

        Layer {
            W: W,
            b: b,
        }
    }

    fn forward(&self, x : &Vec<f64>) -> Option<Vec<f64>>
    {
        let mut output : Vec<f64> = vec!();
        for v in 0..self.W.len() {
            let Wv = &self.W[v];

            if Wv.len() != x.len() {
                return None;
            }

            let mut sum = 0.0;
            for u in 0..Wv.len() {
                sum += Wv[u] * x[u];
            }
            output.push(1.0 / (1.0 + (-sum).exp()));
        }
        return Some(output);
    }
}


/*
use std::fmt;

impl fmt::Display for Creature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "birthday: {}\n", self.birthday);
        write!(f, "colour: {}\n", self.colour);
        write!(f, "size: {}\n", self.size);
        write!(f, "brain:\n{}\n", self.brain)
    }
}

impl fmt::Display for Brain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        for layer in &self.layers {
            write!(f, "{}\n", layer).unwrap();
        }
        write!(f, "\n")
    }
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for v in 0..self.W.len() {
            for u in 0..self.W[v].len() {
                write!(f, "{} ", self.W[v][u]);
            }
            write!(f, "\n");
        }
        write!(f, "\n");
        for u in 0..self.b.len() {
            write!(f, "{} ", self.b[u]);
        }
        write!(f, "\n\n")
    }
}
*/

