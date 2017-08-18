// A WebSocket echo server

extern crate ws;

use ws::{listen, Handler, Sender, Result, Message, Handshake, CloseCode, Error, ErrorKind};
use ws::util::{Token, Timeout};
use std::net::SocketAddr;
use std::collections::HashMap;
use std::sync::{Mutex, Arc};

#[derive(Debug, Copy, Clone)]
struct Player {
    id: u32,
    x: i32,
    y: i32,
}

struct Server {
    out: Sender,
    peer_addr: Option<SocketAddr>,
    players: Arc<Mutex<HashMap<SocketAddr, Player>>>,
}

const STATE: Token = Token(1);

impl Handler for Server {
    fn on_open(&mut self, hs: Handshake) -> Result<()> {
        self.peer_addr = hs.peer_addr;
        let mut players = self.players.lock().unwrap();
        players.insert(hs.peer_addr.unwrap(), Player { id: 0, x: 0, y: 0 });
        self.out.timeout(100, STATE)
    }

    fn on_timeout(&mut self, event: Token) -> Result<()> {
        match event {
            STATE => {
                if let Some(p) = self.players.lock().unwrap().get(&self.peer_addr.unwrap()) {
                    self.out.send(format!("{}:{},{}", p.id, p.x, p.y));
                }
                self.out.timeout(100, STATE)
            }
            _ => Err(Error::new(ErrorKind::Internal, "Invalid token"))
        }
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        let msg = msg.into_text().unwrap();
        println!("Got message: {}", msg);
        let mut split = msg.split(':');
        let command_id = split.next().unwrap().parse::<u32>().unwrap();
        let mut split = split.next().unwrap().split(',');
        let dx = split.next().unwrap().parse::<i32>().unwrap();
        let dy = split.next().unwrap().parse::<i32>().unwrap();
        let mut players = self.players.lock().unwrap();
        if let Some(player) = players.get_mut(&self.peer_addr.unwrap()) {
            if player.id < command_id {
                player.id = command_id;
                player.x += dx;
                player.y += dy;
            }
        }
        Ok(())
    }
}

fn main() {
    let mut players: Arc<Mutex<HashMap<SocketAddr, Player>>> = Arc::new(Mutex::new(HashMap::new()));
    listen("127.0.0.1:3012", |out| { Server { 
        out: out, 
        players: players.clone(),
        peer_addr: None,
    } }).unwrap();
} 
