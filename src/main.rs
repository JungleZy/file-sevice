extern crate core;

mod api;
mod http;
mod routers;
mod service;
mod entity;

fn main() {
    http::http_server::start();
}
