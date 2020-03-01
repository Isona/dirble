// This file is part of Dirble - https://www.github.com/nccgroup/dirble
// Copyright (C) 2019
//  * Izzy Whistlecroft <Izzy(dot)Whistlecroft(at)nccgroup(dot)com>
//  * David Young <David(dot)Young(at)nccgroup(dot)com>
// Released as open source by NCC Group Plc - https://www.nccgroup.com/
//
// Dirble is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Dirble is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Dirble.  If not, see <https://www.gnu.org/licenses/>.

use futures::{future, Future, Poll, Stream};
use hyper::{self, header, Body, Method, Request, Response, StatusCode};
use std::sync::Once;
use std::{thread, time};
use tokio::net::TcpListener;
use tower::{builder::ServiceBuilder, Service};
use tower_hyper::server::Server;

mod basic_requests;
mod scraping;
mod status_detection;

const URL: &str = "http://[::1]:3000";
static START: Once = Once::new();

/* Wrapper around the mock server that launches a thread with it in,
 * using std::sync::Once to make sure only one server is started.
 * Further invocations will block until the first has completed.
 */
pub fn mock_server() {
    START.call_once(|| {
        thread::spawn(|| {
            start_mock_server();
        });
        thread::sleep(time::Duration::from_millis(1000));
    });
}

fn start_mock_server() {
    println!("Making a mock server");
    hyper::rt::run(future::lazy(|| {
        let addr = "[::1]:3000".parse().unwrap();
        let bind = TcpListener::bind(&addr).expect("bind");

        println!("Listening...");

        let maker = ServiceBuilder::new().concurrency_limit(5).service(MakeSvc);
        let server = Server::new(maker);

        bind.incoming()
            .fold(server, |mut server, stream| {
                if let Err(e) = stream.set_nodelay(true) {
                    return Err(e);
                }

                hyper::rt::spawn(
                    server
                        .serve(stream)
                        .map_err(|e| panic!("Server error: {:?}", e)),
                );

                Ok(server)
            })
            .map_err(|e| panic!("server error: {:?}", e))
            .map(|_| {})
    }));
}

fn route(req: Request<Body>) -> Response<Body> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("hi"))
            .unwrap(),
        (&Method::GET, "/301.html") => Response::builder()
            .status(StatusCode::MOVED_PERMANENTLY)
            .header(header::LOCATION, "/301-target.html")
            .body(Body::from("Not here"))
            .unwrap(),
        _ => unimplemented!(),
    }
}

struct Svc;
impl Service<Request<Body>> for Svc {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = future::FutureResult<Self::Response, Self::Error>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        Ok(().into())
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let res: Response<_> = route(req);
        future::ok(res)
    }
}

struct MakeSvc;
impl Service<()> for MakeSvc {
    type Response = Svc;
    type Error = hyper::Error;
    type Future = Box<
        dyn Future<Item = Self::Response, Error = Self::Error> + Send + 'static,
    >;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        Ok(().into())
    }

    fn call(&mut self, _: ()) -> Self::Future {
        Box::new(future::ok(Svc))
    }
}
