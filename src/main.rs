mod api;
mod routers;
mod service;
mod http;

fn main() {
    println!("Hello, world!");
    http::http_server::start();
}
