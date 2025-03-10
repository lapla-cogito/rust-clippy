#![warn(clippy::return_in_conditions)]
#![allow(clippy::blocks_in_conditions)]

fn main() {
    let a = 1;
    if if a == 1 { return } else { true } {}
    //~^ return_in_conditions
}
