extern crate iron;
extern crate router;

use iron::prelude::*;
use iron::status;
use router::Router;
use std::io::Read;

fn main() {
    let mut router = Router::new();

    router.get("/", index, "index");
    router.get("/mine", mine, "mine");
    router.get("/chain", chain, "chain");
    router.get("/nodes/register", nodes_register, "nodes_register");
    router.get("/nodes/resolve", nodes_resolve, "nodes_resolve");
    router.post("/transactions/new", transactions_new, "transactions_new");

    fn index(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "Hello, Blockchain!")))
    }

    fn mine(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "mine")))
    }

    fn chain(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "chain")))
    }

    fn nodes_register(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "nodes_register")))
    }

    fn nodes_resolve(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "nodes_resolve")))
    }

    fn transactions_new(req: &mut Request) -> IronResult<Response> {
        let mut body = String::new();
        req.body
            .read_to_string(&mut body)
            .expect("Failed to read line");

        let res = "transactions_new ".to_string() + &body + &"!".to_string();
        Ok(Response::with((status::Ok, res)))
    }

    Iron::new(router).http("localhost:3000").unwrap();
}
