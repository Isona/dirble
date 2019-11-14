use futures::{future, Future, Poll, Stream};
use hyper::{self, Body, Request, Response};
use tokio::net::TcpListener;
use tower::{builder::ServiceBuilder, Service};
use tower_hyper::server::Server;

mod basic_requests;
mod status_detection;
mod scraping;

const URL: &str = "http://[::1]:3000";

pub fn mock_server()  {
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
        .map_err(|e| panic!("serve errror: {:?}", e))
            .map(|_| {})
    }));
}

struct Svc;
impl Service<Request<Body>> for Svc {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = future::FutureResult<Self::Response, Self::Error>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        Ok(().into())
    }

    fn call(&mut self, _req: Request<Body>) -> Self::Future {
        let res = Response::new(Body::from("Hi"));
        future::ok(res)
    }
}

struct MakeSvc;
impl Service<()> for MakeSvc {
    type Response = Svc;
    type Error = hyper::Error;
    type Future = Box<dyn Future<Item = Self::Response, Error = Self::Error> + Send + 'static>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        Ok(().into())
    }

    fn call(&mut self, _: ()) -> Self::Future {
        Box::new(future::ok(Svc))
    }
}
