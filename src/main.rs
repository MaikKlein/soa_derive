#![feature(proc_macro)]
#[macro_use]
extern crate soa_derive;

#[derive(Debug)]
struct Vec2{
    x: f32,
    y: f32
}

impl Vec2{
    pub fn new() -> Self{
        Vec2{
            x: 0.0,
            y: 0.0
        }
    }
}

#[derive(SoA)]
struct GameObject {
    pos: Vec2,
    vel: Vec2,
    health: f32,
    // Other fields . . .
}

/*
[ derive ( Debug ) ]
struct GameObjectSoA {
    pub pos: Vec<Vec2>,
    pub vel: Vec<Vec2>,
    pub health: Vec<f32>,
}
impl GameObjectSoA {
    pub fn new() -> Self {
        GameObjectSoA {
            pos: Vec::new(),
            vel: Vec::new(),
            health: Vec::new(),
        }
    }
    pub fn push(&mut self, value: GameObject) {
        let GameObject { pos: pos, vel: vel, health: health } = value;
        self.pos.push(pos);
        self.vel.push(vel);
        self.health.push(health);
    }
}
*/

fn main() {
    let mut soa = GameObjectSoA::new();
    let game_object = GameObject {
        pos: Vec2::new(),
        vel: Vec2::new(),
        health: 42.0,
    };
    soa.push(game_object);
    println!("{:?}", soa);
}
