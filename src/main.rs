mod api;
mod http;
mod routers;
mod service;

fn main() {
    http::http_server::start();
}
