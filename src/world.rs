
use std::cmp;
use std::f32;
use std::f64;
use std::fs::File;
use std::io::{ self, Write, Read, ErrorKind };

extern crate rand;
use self::rand::Rng;

extern crate rustc_serialize;
//use self::rustc_serialize::*;


pub type WorldTime = u64;

pub struct World {
    pub run: bool,
    pub time: WorldTime,
    pub terrain: Terrain,
    pub creatures: Vec<Creature>,
    pub total_lives: usize,
}

impl World {
    pub fn new() -> World
    {
        let terrain = Terrain::new();

        let mut creatures : Vec<Creature> = Vec::new();
        for i in 0..CREAT_INIT {
            let colour = i as f32 / CREAT_INIT as f32;

            //let creature = Creature::generate(terrain.size, 1, colour, None);
            //*
            let creature = match Creature::read("creatures/1968.json", terrain.size, 1) {
                Ok(creature) => creature,
                Err(err) => { println!("{}", err); continue; },
            };
            //*/

            creatures.push(creature);
        }

        World {
            run: true,
            time: 1,
            terrain: terrain,
            total_lives: creatures.len(),
            creatures: creatures,
        }
    }

    pub fn timeslice(&mut self)
    {
        if !self.run {
            return;
        }

        self.time += 1;

        self.terrain.timeslice(self.time);

        let mut newcreats : Vec<Creature> = vec!();

        for creature in &mut self.creatures {
            creature.timeslice(&mut self.terrain);

            //if self.time - creature.lastbirth > 1000 && creature.size > 0.75 {
            if self.time - creature.lastbirth > 100 && creature.size > 0.75 && rand::thread_rng().gen_range(0.0, 1.0) <= 0.001 {
                creature.lastbirth = self.time;
                let mut newcreature = creature.spawn(self.time);
                self.total_lives += 1;
                newcreature.position = self.terrain.wrap_position(newcreature.position);
                newcreats.push(newcreature);
            }
        }

        for newcreat in newcreats {
            self.creatures.push(newcreat);
        }

        self.creatures.retain(|ref creature| creature.size >= 0.25);
        //let world_time = self.time;
        //self.creatures.retain(|ref creature| creature.size >= 0.25 && world_time - creature.birthday < 10000);

        if self.creatures.len() <= 0 {
            self.run = false;
        }
    }
}


const WORLD_WIDTH: usize = 200;
const WORLD_HEIGHT: usize = 100;

pub type WorldPoint = [f64; 2];

pub struct Terrain {
    pub size: [usize; 2],
    pub tiles: [[Tile; WORLD_HEIGHT]; WORLD_WIDTH],

    pub season: f64,
    pub season_height: usize,
}

impl Terrain {
    fn new() -> Terrain
    {
        let mut tiles = [[Tile { ttype: 0, food: 0.0 }; WORLD_HEIGHT]; WORLD_WIDTH];
        for col in 0..WORLD_WIDTH {
            for row in 0..WORLD_HEIGHT {
                //tiles[col][row] = Tile::new(rand::thread_rng().gen_range(0, 5));

                let left = if col > 0 { tiles[col - 1][row].ttype } else { rand::thread_rng().gen_range(0, 5) };
                let top = if row > 0 { tiles[col][row - 1].ttype } else { rand::thread_rng().gen_range(0, 5) };
                //let middle = cmp::min((left + top) / 2 + rand::thread_rng().gen_range(0, 2), 5);
                let middle = (left + top) / 2;
                let random = rand::thread_rng().gen_range(0.0, 1.0);

                let ttype = if random <= 0.25 {
                    //cmp::max(middle - 1, 1)
                    if middle > 0 { middle - 1 } else { 1 }
                }
                else if random >= 0.75 {
                    //cmp::min(middle + 1, 5)
                    if middle < 5 { middle + 1 } else { 5 }
                }
                else {
                    middle
                };

                tiles[col][row] = Tile::new(ttype);
                print!("{} ", ttype);
            }
            println!("");
        }

        Terrain {
            size: [ WORLD_WIDTH, WORLD_HEIGHT ],
            tiles: tiles,
            season: 0.0,
            season_height: 0,
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
        const TIMES_PER_YEAR : f64 = 10000.0;
        //self.season = (2.0 * f64::consts::PI * time as f64 / TIMES_PER_YEAR).sin().max(0.0);      // half sine wave period growth, uniform over terrain
        //self.season = (time as f64 % TIMES_PER_YEAR) / TIMES_PER_YEAR;
        self.season = (2.0 * f64::consts::PI * (time as f64 % TIMES_PER_YEAR) / TIMES_PER_YEAR).sin();

        //// Grow new food over time
        if time % 10 == 0 {
            for row in 0..self.size[1] {
                //let season = (f64::consts::PI * (row as f64 / self.size[1] as f64 + self.season / TIMES_PER_YEAR)).sin();           // permanently biases top portion of screen
                let season = (2.0 * f64::consts::PI * ((row as f64 / self.size[1] as f64 + self.season) % 1.0) + (f64::consts::PI / 2.0)).sin().max(0.0);     // window of growth moving upward and wrapping

                for col in 0..self.size[0] {
                    self.tiles[col][row].grow(season);
                }

                if season >= 0.99 {
                    self.season_height = row;
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
    fn new(ttype: i32) -> Tile
    {
        return Tile {
            ttype: ttype,
            food: rand::thread_rng().gen_range(0.0, 100.0),
        };
    }

    fn feed(&mut self) -> f64
    {
        //let r = self.food.min(rand::thread_rng().gen_range(0.1, 1.0));
        //let r = self.food.min(rand::thread_rng().gen_range(0.1, 1.0) * (self.food / 50.0).sqrt());
        let r = self.food.min(self.food.sqrt() * 0.1 * rand::thread_rng().gen_range(0.8, 1.2));
        self.food -= r;
        return r;
    }

    fn grow(&mut self, season: f64)
    {
        //self.food += rand::thread_rng().gen_range(0.0, 1.0) * season;
        self.food += rand::thread_rng().gen_range(0.0, 0.20) * season * (self.ttype as f64).powf(2.0);
        self.food = self.food.min(100.0).max(0.0);
    }
}



const CREAT_INIT : i32 = 100;

pub struct Creature {
    pub id: i32,
    pub colour: f32,
    pub birthday: WorldTime,
    pub lastbirth: WorldTime,
    pub spawns: i32,
    pub eaten: f64,

    pub brain: Brain,

    pub position: [f64; 2],
    pub size: f64,
    pub speed: f64,
    pub angle: f64,
}

static mut last_id : i32 = 0;

impl Creature {
    fn new(x : f64, y: f64, size: f64, speed: f64, angle: f64, birthday: WorldTime, colour: f32, brain: Option<Brain>) -> Creature
    {
        let id = unsafe {
            last_id += 1;
            last_id
        };

        let newbrain = match brain {
            Some(brain) => brain,
            None => Brain::new()
        };

        Creature {
            id: id,
            colour: colour,
            birthday: birthday,
            lastbirth: birthday,
            spawns: 0,
            eaten: 0.0,
            brain: newbrain,

            position: [ x, y ],
            size: size,
            speed: speed + rand::thread_rng().gen_range(-0.2, 0.2),
            angle: angle + rand::thread_rng().gen_range(-0.4, 0.4),
        }
    }

    fn generate(size: [usize; 2], birthday: WorldTime, colour: f32, brain: Option<Brain>) -> Creature
    {
        let position = ( rand::thread_rng().gen_range(0.0, size[0] as f64), rand::thread_rng().gen_range(0.0, size[1] as f64) );
        let size = rand::thread_rng().gen_range(0.75, 1.25);

        return Creature::new(position.0, position.1, size, 0.05, 0.0, birthday, colour, brain);
    }

    fn spawn(&mut self, birthday: WorldTime) -> Creature
    {
        self.spawns += 1;

        let newcolour = self.colour + rand::thread_rng().gen_range(-0.1 as f32, 0.1 as f32).min(1.0).max(0.0);
        //let size = self.size + rand::thread_rng().gen_range(-0.25, 0.25);
        let size = self.size / 2.0;
        self.size -= size;
        return Creature::new(self.position[0] + 2.0, self.position[1] + 2.0, size, self.speed, self.angle, birthday, newcolour, Some(self.brain.spawn()));
    }

    fn timeslice(&mut self, terrain : &mut Terrain)
    {
        //self.x += rand::thread_rng().gen_range(-0.05, 0.05);
        //self.y += rand::thread_rng().gen_range(-0.05, 0.05);

        //self.angle += rand::thread_rng().gen_range(-0.1, 0.1);
        //self.speed += rand::thread_rng().gen_range(-0.001, 0.001);

        let foodbelow = terrain.tiles[self.position[0] as usize][self.position[1] as usize].food;
        let infront = terrain.wrap_position([ self.position[0] + 1.0 * self.angle.cos(), self.position[1] + 1.0 * self.angle.sin() ]);
        let foodahead = terrain.tiles[infront[0] as usize][infront[1] as usize].food;
        let leftfront = terrain.wrap_position([ self.position[0] + 1.0 * (self.angle + f64::consts::PI / 4.0).cos(), self.position[1] + 1.0 * (self.angle + f64::consts::PI / 4.0).sin() ]);
        let foodleft = terrain.tiles[leftfront[0] as usize][leftfront[1] as usize].food;
        let rightfront = terrain.wrap_position([ self.position[0] + 1.0 * (self.angle - f64::consts::PI / 4.0).cos(), self.position[1] + 1.0 * (self.angle - f64::consts::PI / 4.0).sin() ]);
        let foodright = terrain.tiles[rightfront[0] as usize][rightfront[1] as usize].food;

        let input : Vec<f64> = vec!(foodbelow, foodahead, foodleft, foodright, self.size, self.angle, self.speed);
        let output = match self.brain.forward(&input) {
            Some(output) => output,
            None => return,
        };

        //// Continous Control
        //self.angle += (output[0] - 0.5) * 0.1;
        //self.speed += (output[1] - 0.5) * 0.001;

        //// Stepped Control
        self.angle += if output[0] > 0.5 { 0.2 } else if output[1] > 0.5 { -0.2 } else { 0.0 };
        self.speed = if output[2] > 0.5 { 0.2 } else { 0.001 };
        //self.speed = if output[2] > 0.9 { 0.3 } else if output[2] > 0.6 { 0.15 } else { 0.001 };

        /*
        if self.angle < 0.0 {
	    self.angle = 2.0 * f64::consts::PI - self.angle
        }
	self.angle = self.angle % (2.0 * f64::consts::PI);
        */
        self.speed = self.speed.max(0.0).min(1.0);

        self.position[0] += self.speed * self.angle.cos();
        self.position[1] += self.speed * self.angle.sin();
        self.position = terrain.wrap_position(self.position);

        let food = terrain.tiles[self.position[0] as usize][self.position[1] as usize].feed();
        self.eaten += food;
        self.size -= self.size * 0.005;  // cost to live
        self.size += ((1.0 / self.size).powf(2.0) * food * 0.01) - 0.005;
        //self.size += ((1.0 / self.size) * food * 0.01);
    }

    pub fn write(&self, filename: &str) -> Result<(), io::Error>
    {
        let mut f = match File::create(filename) {
            Ok(f) => f,
            Err(err) => return Err(err),
        };

        let encoded = rustc_serialize::json::as_pretty_json(&self.brain);
        match f.write(format!("{}", encoded).as_bytes()) {
            Ok(_) => return Ok(()),
            Err(err) => return Err(err),
        };
    }

    pub fn read(filename: &str, size: [usize; 2], birthday: WorldTime) -> Result<Creature, io::Error>
    {
        let mut f = match File::open(filename) {
            Ok(f) => f,
            Err(err) => return Err(err),
        };

        let mut buffer = String::new();
        match f.read_to_string(&mut buffer) {
            Ok(_) => (),
            Err(err) => return Err(err),
        }

        let brain = match rustc_serialize::json::decode(buffer.as_str()) {
            Ok(brain) => brain,
            Err(err) => { println!("{}", err); return Err(io::Error::new(ErrorKind::InvalidData, "error decoding json")); },
        };

        return Ok(Creature::generate(size, birthday, rand::thread_rng().gen_range(0.0 as f32, 1.0 as f32).min(1.0).max(0.0), brain));
    }
}


const BRAIN_IN : u32 = 7;
const BRAIN_L1 : u32 = 10;
const BRAIN_L2 : u32 = 10;
const BRAIN_OUT : u32 = 3;

#[derive(RustcDecodable, RustcEncodable)]
pub struct Brain {
    layers: Vec<AnyLayer>,
}

impl Brain {
    fn new() -> Brain
    {
        let mut layers : Vec<AnyLayer> = vec!();
        layers.push(FCLayer::new(BRAIN_IN, BRAIN_L1, Activation::Sigmoid));
        layers.push(FCLayer::new(BRAIN_L1, BRAIN_L2, Activation::SinC));
        layers.push(FCLayer::new(BRAIN_L2, BRAIN_OUT, Activation::Tanh));

        Brain {
            layers: layers,
        }
    }

    fn spawn(&self) -> Brain
    {
        let mut layers : Vec<AnyLayer> = vec!();
        for layer in &self.layers {
            layers.push(layer.spawn());
        }

        Brain {
            layers: layers,
        }
    }

    fn forward(&mut self, input : &Vec<f64>) -> Option<Vec<f64>>
    {
        let mut output = input.to_vec();
        for layer in &mut self.layers {
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


#[derive(RustcDecodable, RustcEncodable)]
enum AnyLayer {
    FC(FCLayer),
}

impl AnyLayer {
    fn spawn(&self) -> AnyLayer
    {
        match self {
            &AnyLayer::FC(ref layer) => layer.spawn(),
        }
    }

    fn forward(&mut self, x : &Vec<f64>) -> Option<Vec<f64>>
    {
        match self {
            &mut AnyLayer::FC(ref mut layer) => layer.forward(x),
        }
    }
}


#[derive(Copy, Clone, RustcDecodable, RustcEncodable)]
enum Activation {
    Sigmoid,
    Tanh,
    ReLU,
    Sin,
    SinC,
}

#[inline(always)]
fn activation(formula: Activation, x: f64) -> f64
{
    match formula {
        Activation::Sigmoid => 1.0 / (1.0 + (-x).exp()),
        Activation::Tanh => x.tanh(),
        Activation::ReLU => x.max(0.0),
        Activation::Sin => x.sin(),
        Activation::SinC => if x == 0.0 { 1.0 } else { x.sin() / x },
    }
}


const PARAM_BOUND : f64 = 3.0;

#[allow(non_snake_case)]
#[derive(RustcDecodable, RustcEncodable)]
pub struct FCLayer {
    W: Vec<Vec<f64>>,
    b: Vec<f64>,
    activation: Activation,
}

#[allow(non_snake_case)]
impl FCLayer {

    fn new(width: u32, height: u32, activation: Activation) -> AnyLayer
    {
        let mut W : Vec<Vec<f64>> = vec!();
        for _ in 0..height {
            let mut Wv : Vec<f64> = vec!();
            for _ in 0..width {
                Wv.push(rand::thread_rng().gen_range(-PARAM_BOUND, PARAM_BOUND));
            }
            W.push(Wv);
        }

        let mut b : Vec<f64> = vec!();
        for _ in 0..height {
            b.push(rand::thread_rng().gen_range(-PARAM_BOUND, PARAM_BOUND));
        }

        AnyLayer::FC(FCLayer {
            W: W,
            b: b,
            activation: activation,
        })
    }

    fn spawn(&self) -> AnyLayer
    {
        let mut W : Vec<Vec<f64>> = vec!();
        for v in 0..self.W.len() {
            let mut Wv : Vec<f64> = vec!();
            for u in 0..self.W[v].len() {
                Wv.push((self.W[v][u] + rand::thread_rng().gen_range(-0.4 as f64, 0.4 as f64).powf(3.0)).min(PARAM_BOUND).max(-PARAM_BOUND));
            }
            W.push(Wv);
        }

        let mut b : Vec<f64> = vec!();
        for v in 0..self.b.len() {
            b.push((self.b[v] + rand::thread_rng().gen_range(-0.4 as f64, 0.4 as f64).powf(3.0)).min(PARAM_BOUND).max(-PARAM_BOUND));
        }

        AnyLayer::FC(FCLayer {
            W: W,
            b: b,
            activation: self.activation,
        })
    }

    fn forward(&mut self, x : &Vec<f64>) -> Option<Vec<f64>>
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

            output.push(activation(self.activation, sum));
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

