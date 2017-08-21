// A WebSocket echo server

extern crate ws;

use ws::{listen, Handler, Sender, Result, Message, Handshake, CloseCode, Error, ErrorKind};
use ws::util::{Token, Timeout};
use std::net::SocketAddr;
use std::collections::HashMap;
use std::sync::{Mutex, Arc};
use std::rc::Rc;
use std::cell::Cell;

#[derive(Debug, Copy, Clone)]
struct Player {
    id: u32,
    command_id: u32,
    x: i32,
    y: i32,
}

struct Server {
    out: Sender,
    player_id: u32,
    players: Arc<Mutex<HashMap<u32, Player>>>,
}

const STATE: Token = Token(1);

impl Handler for Server {
    fn on_open(&mut self, hs: Handshake) -> Result<()> {
        let mut players = self.players.lock().unwrap();
        players.insert(self.player_id, Player { id: self.player_id, command_id: 0, x: 0, y: 0 });
        self.out.send(format!("welcome:{}", self.player_id));
        self.out.timeout(100, STATE)
    }

    fn on_timeout(&mut self, event: Token) -> Result<()> {
        match event {
            STATE => {
                let msg = self.players.lock().unwrap().values()
                    .map(|p| format!("{}:{}:{},{}", p.id, p.command_id, p.x, p.y))
                    .collect::<Vec<String>>().join("\n");
                self.out.send(msg);
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
        if let Some(player) = players.get_mut(&self.player_id) {
            if player.command_id < command_id {
                player.command_id = command_id;
                player.x += dx;
                player.y += dy;
            }
        }
        Ok(())
    }

    fn on_close(&mut self, _code: CloseCode, reason: &str) {
        self.players.lock().unwrap().remove(&self.player_id).unwrap();
    }
}

fn main() {
    let mut players: Arc<Mutex<HashMap<u32, Player>>> = Arc::new(Mutex::new(HashMap::new()));
    let mut new_player_id = Rc::new(Cell::new(0_u32));
    listen("127.0.0.1:3012", |out| {
        let new_player_id = new_player_id.clone();
        let player_id = new_player_id.get();
        new_player_id.set(player_id + 1);
        Server { 
            out: out, 
            players: players.clone(),
            player_id: player_id,
        } 
    }).unwrap();
} 
