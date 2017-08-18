// A WebSocket echo server

extern crate ws;

use std::rc::Rc;
use std::cell::Cell;

use ws::{listen, Handler, Sender, Result, Message, Handshake, CloseCode, Error, ErrorKind};
use ws::util::{Token, Timeout};

#[derive(Debug, Copy, Clone)]
struct Position {
    id: u32,
    x: i32,
    y: i32,
}

struct Server {
    out: Sender,
    position: Position,
}

const STATE: Token = Token(1);

impl Handler for Server {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        self.out.timeout(100, STATE)
    }

    fn on_timeout(&mut self, event: Token) -> Result<()> {
        match event {
            STATE => {
                let p = &mut self.position;
                self.out.send(format!("{}:{},{}", p.id, p.x, p.y));
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
        if self.position.id < command_id {
            self.position.id = command_id;
            self.position.x += dx;
            self.position.y += dy;
        }
        Ok(())
    }
}

fn main() {
    // let position = Rc::new(Cell::new(Position { id: 0, x: 0, y: 0 }));
    listen("127.0.0.1:3012", |out| { Server { 
        out: out, 
        position: Position { 
            id: 0, 
            x: 0, 
            y: 0 
        } 
    } }).unwrap();
} 
