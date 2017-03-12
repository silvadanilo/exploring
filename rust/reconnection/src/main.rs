extern crate futures;
extern crate tokio_core;
extern crate tokio_line;

use futures::future::{self, Future, Loop};
use futures::{Stream};
use std::{io, str};
use tokio_core::io::{Io};
use tokio_core::net::{TcpStream};
use tokio_core::reactor::{Core, Handle};
use tokio_line::LineCodec;
use std::{thread, time};

fn get_connection(handle: &Handle) -> Box<Future<Item = (), Error = io::Error>> {
    let remote_addr = "127.0.0.1:9876".parse().unwrap();
    let tcp = TcpStream::connect(&remote_addr, handle);

    let client = tcp.and_then(move |stream| {
        let (_sender, receiver) = stream.framed(LineCodec).split();
        let reader = receiver
            .for_each(|message| {
                println!("{}", message);
                Ok(())
            })
            .and_then(|_| {
                println!("CLIENT DISCONNECTED");
                Ok(())
            });
        reader
    });

    let client = client
        .or_else(|_| {
            println!("connection refuse");
            Err(io::Error::new(io::ErrorKind::Other, "connection refuse"))
        });

    Box::new(client)
}

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let client = future::loop_fn((), |_| {
        get_connection(&handle)
            .map(|_| -> Loop<(), ()> {
                Loop::Continue(())
            })
            .or_else(|_| -> Result<Loop<(), ()>, ()> {
                thread::sleep(time::Duration::from_millis(100));
                Ok(Loop::Continue(()))
            })
    });

    core.run(client).unwrap();
}
