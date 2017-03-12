extern crate futures;
extern crate tokio_core;
extern crate tokio_line;

use futures::future::{self, Future, Loop};
use futures::{Stream, Sink};
use futures::sync::mpsc::{self, UnboundedSender, UnboundedReceiver};
use tokio_core::io::{Io};
use tokio_core::net::{TcpStream};
use tokio_core::reactor::{Core, Handle};
use tokio_line::LineCodec;
use std::{io, str};
use std::{thread, time};


fn send_data_to_remote_server(handle: &Handle, bufrx: UnboundedReceiver<String>) -> Box<Future<Item = UnboundedReceiver<String>, Error = io::Error>> {
    let remote_addr = "127.0.0.1:9876".parse().unwrap();
    let tcp = TcpStream::connect(&remote_addr, handle);

    let client = tcp.and_then(move |stream| {
        let (sender, receiver) = stream.framed(LineCodec).split();
        let reader = receiver
            .for_each(|message| {
                println!("{}", message);
                Ok(())
            })
            .and_then(|bufrx| {
                println!("CLIENT DISCONNECTED");
                /***************** START FIXME ************/
                let (_, bufrx) = mpsc::unbounded(); //FIXME: `bufrx` received in a clousure is actually an `()` so I can't use it, I put this here only as sample and to satisfy the compiler
                /***************** END FIXME ************/
                Ok(bufrx)
            });

        let writer = bufrx
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "error kind returned should be the same of `sender` sink in `forward()`"))
            .forward(sender)
            .and_then(|(bufrx, sender)| {
                // bufrx
                /***************** START FIXME ************/
                let (_, bufrx) = mpsc::unbounded(); //FIXME: `bufrx` received in a clousure is actually an `()` so I can't use it, I put this here only as sample and to satisfy the compiler
                /***************** END FIXME ************/
                Ok(bufrx)
            });

        reader.select(writer)
            .map(|(bufrx, nf)| {
                bufrx
            })
            .map_err(|(err, nf)| {
                err
            })

        // let (_, bufrx) = mpsc::unbounded();
        // reader.map(|_| bufrx).select(writer.map(|_| bufrx))
    }).or_else(|_| {
        println!("connection refuse");
        Err(io::Error::new(io::ErrorKind::Other, "connection refuse"))
    });

    Box::new(client)
}

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let (buftx, bufrx) = mpsc::unbounded();

    let client = future::loop_fn(bufrx, |bufrx| {
        send_data_to_remote_server(&handle, bufrx)
            .map(|bufrx| -> Loop<UnboundedReceiver<String>, UnboundedReceiver<String>> {
                /***************** START FIXME ************/
                // actually the `bufrx` received in this clouser is not the original one, but I
                // think that the fix should be made in the `send_data_to_remote_server()` method
                /***************** END FIXME ************/
                Loop::Continue(bufrx)
            })
            .or_else(|err| -> Result<Loop<UnboundedReceiver<String>, UnboundedReceiver<String>>, ()> {
                thread::sleep(time::Duration::from_millis(50));
                /***************** START FIXME ************/
                let (_, bufrx) = mpsc::unbounded(); //FIXME: argument received in a clousure is actually an `io::Error` and I lost the ownership of the original bufrx so I can't use it, I put this here only as sample and to satisfy the compiler
                /***************** END FIXME ************/

                Ok(Loop::Continue(bufrx))
            })
    });

    core.run(client).unwrap();
}
