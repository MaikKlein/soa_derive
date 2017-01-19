#![allow(dead_code)]
#[macro_use]
extern crate vec_derive;
#[macro_use]
extern crate vec_macros;
extern crate num;
#[macro_use]
extern crate quote;
use num::{Num, Float, Zero, NumCast};


#[derive(Copy, Clone, Debug, Vector)]
#[repr(C)]
struct Vec2<T> {
    x: T,
    y: T,
}

#[derive(Copy, Clone, Debug, Vector)]
#[repr(C)]
struct Vec3<T> {
    x: T,
    y: T,
    z: T,
}

#[derive(Copy, Clone, Debug, Vector)]
#[repr(C)]
struct Vec4<T> {
    x: T,
    y: T,
    z: T,
    w: T,
}

trait Tokenize {
    fn tokenize(&self) -> quote::Tokens;
}

impl Tokenize for Vec<quote::Tokens> {
    fn tokenize(&self) -> quote::Tokens {
        let mut tokens = quote::Tokens::new();
        tokens.append_all(self.iter());
        tokens
    }
}

fn main() {
    let v = Vec3::new(1.0, 2.0, 3.0);
    let v1 = v.map(|f| f * 2.0);
    println!("{:?}", v1);
}
