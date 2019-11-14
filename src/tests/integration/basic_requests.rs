use futures::{future, Future, Poll, Stream};
use hyper::{self, Body, Request, Response};
use std::sync::Arc;
use std::{thread, time};
use tokio::net::TcpListener;
use tower::{builder::ServiceBuilder, Service};
use tower_hyper::server::Server;
use url::Url;

use super::mock_server;
use crate::arg_parse::GlobalOpts;
use crate::request;

#[test]
fn test_200_response() {
    thread::spawn(|| {
        mock_server();
    });

    thread::sleep(time::Duration::from_millis(1000));

    let mut global_opts: GlobalOpts = Default::default();
    let global_opts = Arc::new(global_opts);
    let mut easy = request::generate_easy(&global_opts);

    let url = Url::parse(super::URL).unwrap();

    let rr = request::make_request(&mut easy, url);

    println!("{:?}", rr);

    panic!();
}
