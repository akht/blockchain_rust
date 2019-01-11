extern crate iron;
extern crate router;

use iron::prelude::*;
use iron::status;
use router::Router;

fn main() {
    let mut router = Router::new();

    router.get("/", index, "index");

    Iron::new(router).http("localhost:3000").unwrap();

    fn index(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "Hello, Blockchain!")))
    }
}
