#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate futures;
extern crate tokio_core;
extern crate hyper;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

mod mediator;
mod server;
mod model;
mod ai;

pub use server::start;
