extern crate crypto;
extern crate iron;
extern crate router;
extern crate rustc_serialize;
extern crate serde;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use iron::prelude::*;
use iron::status;
use iron::Handler;
use router::Router;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

fn main() {
    let blockchain = Blockchain::new();
    println!("{:?}", blockchain);

    let handlers = Handlers::new(blockchain);

    let mut router = Router::new();
    router.get("/mine", handlers.mine, "mine");
    router.get("/chain", handlers.chain, "chain");
    router.get("/nodes/register", nodes_register, "nodes_register");
    router.get("/nodes/resolve", nodes_resolve, "nodes_resolve");
    router.post(
        "/transactions/new",
        handlers.transactions_new,
        "transactions_new",
    );

    fn nodes_register(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "nodes_register")))
    }

    fn nodes_resolve(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "nodes_resolve")))
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

        let mut last_block = self.chain[&self.chain.len() - 1].clone();
        last_block.index += 1;

        let index = last_block.index;

        self.chain.remove(&self.chain.len() - 1);
        self.chain.push(last_block);

        index
    }

    fn proof_ow_work(&mut self, last_proof: &usize) -> usize {
        let mut proof = 0;

        while !self.valid_proof(last_proof, &mut proof) {
            proof += 1;
        }

        proof
    }

    fn valid_proof(&self, last_proof: &usize, proof: &usize) -> bool {
        let guess = vec![last_proof.to_string(), proof.to_string()].concat();

        let mut sha256 = Sha256::new();
        sha256.input_str(&guess);
        let guess_hash = sha256.result_str();
        let prefix = String::from("0000");

        guess_hash[0..4] == prefix[..]
    }

    fn hash(&self, block: &Block) -> String {
        let mut sha256 = Sha256::new();
        sha256.input_str(&serde_json::to_string(block).unwrap());

        sha256.result_str()
    }
}

#[derive(Clone, Debug, Serialize)]
struct Block {
    index: usize,
    timestamp: SystemTime,
    transactions: Vec<Transaction>,
    proof: usize,
    previous_hash: String,
}

#[derive(Clone, Serialize, Debug)]
struct Transaction {
    sender: String,
    recipient: String,
    amount: usize,
}

struct Handlers {
    chain: ChainHandler,
    mine: MineHandler,
    transactions_new: TransactionsNewHandler,
}

impl Handlers {
    fn new(blockchain: Blockchain) -> Handlers {
        let blockchain = Arc::new(Mutex::new(blockchain));
        Handlers {
            chain: ChainHandler::new(blockchain.clone()),
            mine: MineHandler::new(blockchain.clone()),
            transactions_new: TransactionsNewHandler::new(blockchain.clone()),
        }
    }
}

struct ChainHandler {
    blockchain: Arc<Mutex<Blockchain>>,
}

impl ChainHandler {
    fn new(blockchain: Arc<Mutex<Blockchain>>) -> ChainHandler {
        ChainHandler { blockchain }
    }
}

impl Handler for ChainHandler {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        let chain = &self.blockchain.lock().unwrap().chain;

        let response = json!({
            "chain": chain,
            "length": &chain.len(),
        });

        Ok(Response::with((status::Ok, response.to_string())))
    }
}

struct MineHandler {
    blockchain: Arc<Mutex<Blockchain>>,
}

impl MineHandler {
    fn new(blockchain: Arc<Mutex<Blockchain>>) -> MineHandler {
        MineHandler { blockchain }
    }
}

impl Handler for MineHandler {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        let blockchain = &mut self.blockchain.lock().unwrap();
        let last_block = blockchain.chain[&blockchain.chain.len() - 1].clone();
        let last_proof = &last_block.proof;
        let proof = blockchain.proof_ow_work(last_proof);

        blockchain.new_transaction(String::from("0"), String::from("A"), 1);

        let block = blockchain.new_block(proof, None);

        let response = json!({
            "message": String::from("新しいブロックを採掘しました"),
            "index": block.index,
            "transactions": block.transactions,
            "proof": block.proof,
            "previous_hash": block.previous_hash,
        });

        Ok(Response::with((status::Ok, response.to_string())))
    }
}

struct TransactionsNewHandler {
    blockchain: Arc<Mutex<Blockchain>>,
}

impl TransactionsNewHandler {
    fn new(blockchain: Arc<Mutex<Blockchain>>) -> TransactionsNewHandler {
        TransactionsNewHandler { blockchain }
    }
}

impl Handler for TransactionsNewHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let blockchain = &mut self.blockchain.lock().unwrap();

        let mut body = String::new();
        req.body
            .read_to_string(&mut body)
            .expect("Failed to read line");

        let values: Value = serde_json::from_str(&body).unwrap();

        let index = blockchain.new_transaction(
            values["sender"].to_string(),
            values["recipient"].to_string(),
            values["amount"].to_string().parse::<usize>().unwrap(),
        );

        let message = "トランザクションはブロック ".to_string()
            + &index.to_string()
            + " に追加されました";

        let response = json!({
            "message": message,
        });

        Ok(Response::with((status::Ok, response.to_string())))
    }
}
