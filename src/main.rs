mod api;
mod routers;
mod service;
mod http;

fn main() {
    http::http_server::start();
}
