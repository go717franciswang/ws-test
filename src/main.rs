// A WebSocket echo server

extern crate ws;
extern crate nalgebra as na;
extern crate ncollide;

use ws::{listen, Handler, Sender, Result, Message, Handshake, CloseCode, Error, ErrorKind};
use ws::util::{Token, Timeout};
use std::collections::HashMap;
use std::sync::{Mutex, Arc};
use std::rc::Rc;
use std::cell::Cell;
use na::{Isometry2, Vector2};
use ncollide::shape::{Cuboid, Ball};
use ncollide::query;
use std::sync::mpsc::{channel, self};
use std::thread;
use std::time;

#[derive(Debug, Copy, Clone)]
struct Player {
    id: u32,
    command_id: u32,
    x: i32,
    y: i32,
}

#[derive(Copy, Clone)]
struct Move {
    player_id: u32,
    command_id: u32,
    dx: i32,
    dy: i32,
}

struct Server {
    out: Sender,
    player_id: u32,
    players: Arc<Mutex<HashMap<u32, Player>>>,
    sender: mpsc::Sender<Move>,
}

const STATE: Token = Token(1);

impl Handler for Server {
    fn on_open(&mut self, _hs: Handshake) -> Result<()> {
        let mut players = self.players.lock().unwrap();
        players.insert(self.player_id, Player { id: self.player_id, command_id: 0, x: 0, y: 0 });
        self.out.send(format!("welcome:{}", self.player_id)).unwrap();
        self.out.timeout(100, STATE)
    }

    fn on_timeout(&mut self, event: Token) -> Result<()> {
        match event {
            STATE => {
                let msg = self.players.lock().unwrap().values()
                    .map(|p| format!("{}:{}:{},{}", p.id, p.command_id, p.x, p.y))
                    .collect::<Vec<String>>().join("\n");
                self.out.send(msg).unwrap();
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
        self.sender.send(Move {
            player_id: self.player_id,
            command_id: command_id,
            dx: dx,
            dy: dy,
        }).unwrap();
        Ok(())
    }

    fn on_close(&mut self, _code: CloseCode, _reason: &str) {
        self.players.lock().unwrap().remove(&self.player_id).unwrap();
    }
}

fn bound_move(players: &HashMap<u32, Player>, m: &mut Move) {
    if let Some(player) = players.get(&m.player_id) {
        for p in players.values() {
            if p.id == player.id {
                continue;
            }

            let x = player.x + m.dx;
            let y = player.y + m.dy;
            if (x - p.x).abs() < 10 && (player.y - p.y).abs() < 10 {
                m.dx = 0;
            }
            if (player.x - p.x).abs() < 10 && (y - p.y).abs() < 10 {
                m.dy = 0;
            }
        }
    }
}

fn main() {
    let players: Arc<Mutex<HashMap<u32, Player>>> = Arc::new(Mutex::new(HashMap::new()));

    let (sender, receiver) = channel::<Move>();
    let frame_ms = time::Duration::from_millis(20);
    let engine_players = players.clone();
    thread::spawn(move|| {
        loop {
            while let Ok(mut m) = receiver.try_recv() {
                let mut players = engine_players.lock().unwrap();
                bound_move(&players, &mut m);

                if let Some(player) = players.get_mut(&m.player_id) {
                    if player.command_id < m.command_id {
                        player.command_id = m.command_id;
                        player.x += m.dx;
                        player.y += m.dy;
                    }
                }
            }
            
            thread::sleep(frame_ms);
        }
    });

    let new_player_id = Rc::new(Cell::new(0_u32));
    listen("127.0.0.1:3012", |out| {
        let new_player_id = new_player_id.clone();
        let player_id = new_player_id.get();
        new_player_id.set(player_id + 1);
        Server { 
            out: out, 
            players: players.clone(),
            player_id: player_id,
            sender: sender.clone(),
        } 
    }).unwrap();
} 
