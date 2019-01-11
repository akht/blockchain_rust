extern crate iron;
extern crate router;
extern crate crypto;
extern crate rustc_serialize;

use iron::prelude::*;
use iron::status;
use router::Router;
use std::io::Read;
use std::collections::HashSet;
use std::time::SystemTime;
use rustc_serialize::hex::ToHex;
use crypto::digest::Digest;
use crypto::sha2::Sha256;

fn main() {
    let blockchain = Blockchain::new();
    println!("{:?}", blockchain);



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

#[derive(Debug)]
struct Blockchain {
    chain: Vec<Block>,
    nodes: HashSet<String>,
    current_transactions: Vec<Transaction>,
}

impl Blockchain {
    fn new() -> Blockchain {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            nodes: HashSet::new(),
            current_transactions: Vec::new(),
        };
        blockchain.new_block(100, Some(String::from("1")));

        blockchain
    }

    fn new_block(&mut self, proof: usize, previous_hash: Option<String>) -> Block {
        let block = Block {
            index: self.chain.len() + 1,
            timestamp: SystemTime::now(),
            transactions: self.current_transactions.to_vec(),
            proof: proof,
            previous_hash: match previous_hash {
                Some(hash) => hash,
                None => self.hash(&self.chain[self.chain.len() - 1]),
            },
        };

        let clone = block.clone();

        self.current_transactions = Vec::new();
        self.chain.push(block);

        clone
    }

    fn new_transaction(&mut self, sender: String, recipient: String, amount: usize) -> usize {
        self.current_transactions.push({
            Transaction {
                sender: sender,
                recipient: recipient,
                amount: amount,
            }
        });

        let mut last_block = self.last_block();
        last_block.index += 1;
        last_block.index
    }

    fn last_block(self) -> Block {
        // let clone = self.chain[self.chain.len() - 1].clone();
        // clone

        self.chain[self.chain.len() - 1]
    }

    fn hash(&self, block: &Block) -> String {
        let mut sha256 = Sha256::new();
        sha256.input_str("hoge");

        sha256.result_str().as_bytes().to_hex()
    }
}


#[derive(Clone, Debug)]
struct Block {
    index: usize,
    timestamp: SystemTime,
    transactions: Vec<Transaction>,
    proof: usize,
    previous_hash: String,
}

#[derive(Clone, Debug)]
struct Transaction {
    sender: String,
    recipient: String,
    amount: usize,
}
