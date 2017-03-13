extern crate futures;
extern crate tokio_core;
extern crate tokio_line;

use futures::future::{self, Future, Loop, IntoFuture};
use futures::{Stream, Sink};
use futures::sync::mpsc::{self, UnboundedSender, UnboundedReceiver};
use tokio_core::io::{Io};
use tokio_core::net::{TcpStream};
use tokio_core::reactor::{Core, Handle, Timeout};
use tokio_line::LineCodec;
use std::{io, str};
use std::{thread, time};
use std::time::Duration;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

struct RemoteConnections {
    connections: HashMap<String, UnboundedSender<String>>,
    handle: Handle,
}

impl RemoteConnections {
    fn new(handle: Handle) -> Self {
        RemoteConnections {
            connections: HashMap::<String, UnboundedSender<String>>::new(),
            handle: handle,
        }
    }

    fn add(&mut self, tx: UnboundedSender<String>) {
        self.connections.insert("prova".to_string(), tx);
    }

    // fn send(&self, message: String) -> Box<Future<Item = (), Error = ()>> {
    fn send(&self, message: String) -> Box<Result<(), ()>> {
        // let f = for (_key, tx) in self.connections.borrow_mut().iter_mut() {
        //     let f = tx.send(message.clone())
        //         .map(|_| ())
        //         .map_err(|_| ());

        //     f
        //     // self.handle.spawn(f);
        // };

        let tx = self.connections.get("prova").unwrap();
        let f = tx.send(message.clone())
            .map(|_| ())
            .map_err(|_| ());

        return Box::new(f);
    }

    fn new_connection(&self) -> UnboundedReceiver<String> {
        let (remote_tmp_tx, remote_tmp_rx) = mpsc::unbounded::<String>();
        remote_tmp_rx
    }
}


fn send_data_to_remote_server<'a>(handle: &Handle, connections: Rc<RefCell<RemoteConnections>>) -> Box<Future<Item = (), Error = io::Error>> {
    let remote_addr = "127.0.0.1:9876".parse().unwrap();
    let tcp = TcpStream::connect(&remote_addr, handle);

    let client = tcp.and_then(move |stream| {
        let (remote_tmp_tx, remote_tmp_rx) = mpsc::unbounded::<String>();
        connections.borrow_mut().add(remote_tmp_tx);

        let (sender, receiver) = stream.framed(LineCodec).split();
        let reader = receiver
            .for_each(|message| {
                println!("{}", message);
                Ok(())
            })
            .and_then(|bufrx| {
                println!("CLIENT DISCONNECTED");
                Ok(())
            });

        // reader

        let writer = remote_tmp_rx
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "error kind returned should be the same of `sender` sink in `forward()`"))
            .forward(sender)
            .and_then(|(rx, tx)| Ok(()))
        ;

        // let writer = bufrx
        //     .map_err(|_| io::Error::new(io::ErrorKind::Other, "error kind returned should be the same of `sender` sink in `forward()`"))
        //     .forward(sender)
        //     .and_then(|(bufrx, sender)| {
        //         Ok(())
        //     });

        reader.select(writer)
            .map(|(bufrx, nf)| {
                ()
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

    let connections = Rc::new(RefCell::new(RemoteConnections::new(handle.clone())));

    let (buftx, bufrx) = mpsc::unbounded();
    simulated_messaging_receiving_from_clients(buftx.clone(), &handle.clone());

    let connections_inner = connections.clone();
    let f = bufrx.for_each(move |message| {
        connections_inner.borrow().send(message.clone());
        Ok(())
    }).map_err(|_| ());
    handle.spawn(f);

    let client = future::loop_fn((), |_| {
        send_data_to_remote_server(&handle, connections.clone())
            .map(|_| -> Loop<(), ()> {
                Loop::Continue(())
            })
            .or_else(|err| -> Result<Loop<(), ()>, ()> {
                thread::sleep(time::Duration::from_millis(50));
                Ok(Loop::Continue(()))
            })
    });

    core.run(client).unwrap();
}

fn simulated_messaging_receiving_from_clients(buftx: UnboundedSender<String>, handle: &Handle) -> () {
    for i in 1..11 {
        let buftxcloned = buftx.clone();
        let handle2 = handle.clone();
        let t = Timeout::new(Duration::new(i, 0), &handle).into_future().flatten();
        let ft = t.and_then(move |_| {
            println!("Timed out");
            let a = buftxcloned
                .send(format!("Messagio {}", i).to_string())
                .map(|_| ())
                .map_err(|_| ())
            ;
            handle2.spawn(a);
            Ok(())
        });
        handle.spawn(ft.map_err(|_| ()));
    };
}
