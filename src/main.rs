extern crate redis;
extern crate serialize;
extern crate iron;
extern crate http;
extern crate session;

use std::io::net::ip::{SocketAddr, Ipv4Addr};
use iron::{Iron, Chain, Request, Response, IronResult, ChainBuilder};
use iron::status;
use session::sessions::RequestSession;
use session::{Session, Sessions};

mod redis_session;

#[allow(dead_code)]
pub struct Organization<'r> {
  name: String,
  api_keys: Vec<String>,
  users: Vec<User<'r>>,
}

#[allow(dead_code)]
pub struct User<'r> {
  username: String,
  org: Organization<'r>,
  password: &'r[u8],
}

fn get_count(req: &mut Request) -> IronResult<Response> {
    // Retrieve our session from the store
    let session = req.extensions.find_mut::<RequestSession, Session<String, u32>>().unwrap();
    // Store or increase the sessioned count
    let count = session.upsert(1u32, |v: &mut u32| { *v = *v + 1; } );

    println!("{} hits from\t{}", count, req.remote_addr.unwrap())

    Ok(Response::with(::http::status::Ok, format!("Sessioned count: {}", count).as_slice()))
}

fn main() {
    let mut chain = ChainBuilder::new(get_count);
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    chain.link_before(Sessions::new(key_from_socket_addr, redis_session::RedisSessionStore::<String, u32>::new(client)));
    Iron::new(chain).listen(Ipv4Addr(127, 0, 0, 1), 4010);
    println!("Starting patina on port 4010");
}

fn key_from_socket_addr(req: &Request) -> String {
  req.remote_addr.unwrap().to_string()
}

