extern crate futures;
extern crate tokio_core;
extern crate tokio_line;

use futures::{Async, AsyncSink, StartSend, Poll, Stream, Sink, IntoFuture, Future, future};
use futures::sync::mpsc::{SendError};
use futures::Async::{NotReady, Ready};
use futures::sync::mpsc::UnboundedReceiver;
use futures::sync::mpsc;
use futures::task::{self, Task};
use std::time::Duration;
use std::cell::RefCell;
use std::rc::Rc;
use tokio_core::reactor::{Core, Timeout, Handle};
use tokio_core::net::{TcpStream, TcpListener};
use tokio_core::io::{Io, Codec, EasyBuf, Framed};
use tokio_line::LineCodec;
use std::io::{self, Read, Write, ErrorKind};
use std::{thread, time};


#[derive(Clone)]
struct Buffer {
    remote_server_is_ready: bool,
    remote_tx: Option<mpsc::Sender<String>>,
    foo: u8,
    b: Vec<String>,
    handle: Handle,
}

impl Buffer {
    fn ready(&mut self) {
        self.remote_server_is_ready = true;
        println!("READY");
        println!("R {}", self.foo);
    }

    fn set_foo(&mut self, foo: u8) {
        self.foo = foo;
    }

    fn increment_foo(&mut self) {
        self.foo = self.foo + 1;
    }

    fn get_connection(&self, myrx: UnboundedReceiver<String>) -> Box<Future<Item = (), Error = io::Error>> {
        let remote_addr = "127.0.0.1:9876".parse().unwrap();
        let tcp = TcpStream::connect(&remote_addr, &self.handle);

        let client = tcp.and_then(move |stream| -> Box<Future<Item = (), Error = io::Error>> {
            let (sender, receiver) = stream.framed(LineCodec).split();
            let reader = receiver.for_each(|message| {
                println!("{}", message);
                Ok(())
            })
            .map(|_| -> Result<(), ()> {
                Ok(())
            })
            .map_err(|_| -> Result<(), ()> {
                Ok(())
            });

            let writer = myrx
                .map_err(|_| ())
                .fold(sender, |sender, msg| {
                    sender.send(msg).map_err(|_| ())
                })
                .map(|_| -> Result<(), ()> {
                    Ok(())
                })
                .map_err(move |_| -> Result<(), ()> {
                    Ok(())
                })
            ;

            // let writer = myrx.forward(Buffer{
            //     b: vec![],
            //     foo:1,
            //     handle:self.handle.clone(),
            //     remote_server_is_ready: false,
            //     remote_tx: None,
            // });
            // self.handle.spawn(writer.map(|_| {
            //     ()
            // }));

            reader.select(writer)
                .map(|_| {
                    println!("CLIENT DISCONNECTED");
                    ()
                })
                .map_err(|_| {
                    io::Error::new(io::ErrorKind::Other, "cl disc");
                })
        });

        let client = client
            .or_else(|_| {
                println!("connection refuse");
                thread::sleep(time::Duration::from_millis(100));
                Ok(())
                // Err(io::Error::new(io::ErrorKind::Other, "connection refuse"))
            });

        Box::new(client)
    }
}

impl Sink for Buffer {
    type SinkItem = String;
    type SinkError = ();

    fn start_send(&mut self, msg: String) -> StartSend<String, ()> {

        let (tx, rx) = mpsc::unbounded();

        let connection = self.get_connection(rx)
            .and_then(|_| {
                println!("YEAH!");
                Ok(())
            });
        self.handle.spawn(connection.map_err(|_| ()));
        // self.increment_foo();
        // if self.foo < 3 {
        //     println!("NOT READY");
        //     println!("NR {}", self.foo);
        //     println!("message is {}", msg);
        //     return Ok(AsyncSink::NotReady(msg));
        // }

        println!("");
        println!("********");
        println!("start send");
        println!("is ready = {}", self.remote_server_is_ready);
        // Ok(AsyncSink::NotReady(msg))
        self.b.push(msg);
        // Ok(AsyncSink::Ready)

        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), ()> {
        println!("");
        println!("********");
        println!("complete: {:?}", self.b);
        println!("is ready = {}", self.remote_server_is_ready);


        // let t = Timeout::new(Duration::new(5, 0), &handle).into_future().flatten();
        // let f = t.and_then(move |_| {
        //     let mut bo = rc_buffer_cloned.borrow_mut();
        //     bo.ready();
        //     Ok(())
        // });


        // Ok(Async::NotReady)
        Ok(Async::Ready(()))
    }
}

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let mut buffer = Buffer {
        remote_server_is_ready: false,
        remote_tx: None,
        foo: 1,
        b: Vec::new(),
        handle: handle.clone(),
    };

    // let rc_buffer = Rc::new(RefCell::new(buffer));
    // let rc_buffer_cloned = rc_buffer.clone();

    // let t = Timeout::new(Duration::new(5, 0), &handle).into_future().flatten();
    // let f = t.and_then(move |_| {
    //     let mut bo = rc_buffer_cloned.borrow_mut();
    //     bo.ready();
    //     Ok(())
    // });

    // let f = f.map_err(|_| panic!());
    // handle.spawn(f);











    let (tx, rx) = mpsc::unbounded();
    let forward_future = rx.forward(buffer);
    handle.spawn(forward_future.map(|_| {
        ()
    }));

    let address = "0.0.0.0:12345".parse().unwrap();
    let listener = TcpListener::bind(&address, &core.handle()).unwrap();
    let connections = listener.incoming();

    // let sync = rx.for_each(move |msg| {
    //     clients_clone.broadcast(msg)
    // });
    // handle.spawn(sync);

    let server = connections.for_each(|(socket, _)| {
        let transport = socket.framed(LineCodec);

        let nonhocapitoperchedevoclonarlo = tx.clone();
        let process_connection = transport.for_each(move |line| {
            nonhocapitoperchedevoclonarlo.clone().send(line)
                .map_err(|err| io::Error::new(ErrorKind::Other, err))
                .map(|_| ())
        });

        handle.spawn(process_connection.map_err(|_| ()));

        Ok(())
    });

    core.run(server).unwrap();
}
