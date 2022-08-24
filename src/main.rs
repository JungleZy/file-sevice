mod api;
mod http;
mod routers;
mod service;


fn main() {
    unsafe { http::http_server::start(); }
}
