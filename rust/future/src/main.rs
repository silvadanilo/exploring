#[macro_use]
extern crate log;
extern crate env_logger;
extern crate futures;
extern crate tokio_core;
extern crate tokio_line;

use futures::{Async, AsyncSink, StartSend, Poll, Stream, Sink, IntoFuture, Future, future};
use futures::task::{self, Task};
use tokio_core::reactor::{Core, Timeout, Handle};
// use futures::sync::mpsc::{SendError};
// use futures::Async::{NotReady, Ready};
// use futures::sync::mpsc::UnboundedReceiver;
// use futures::sync::mpsc;
// use futures::task::{self, Task};
use std::time::Duration;
use std::{thread, time};
use std::io::{self, Read, Write, ErrorKind};
use std::cell::RefCell;
use std::rc::Rc;
use tokio_core::net::{TcpStream, TcpListener};
use tokio_line::LineCodec;
use tokio_core::io::{Io};
use futures::sync::mpsc;
use futures::sync::oneshot;


struct Sample {
    x: i8,
    handle: Handle,
    task: Option<Task>,
    foo: Option<i8>,
    rx: oneshot::Receiver<i8>,
}

impl Sample {
    fn set_foo(&mut self, foo: i8) {
        self.foo = Some(foo);
        if let Some(ref task) = self.task {
            task.unpark();
        }
    }
}

impl Future for Sample {
    type Item = i8;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        println!("Starting poll");

        self.rx.poll()
            .map_err(|err| io::Error::new(ErrorKind::Other, err))

        // match self.rx.poll().expect("non puo fallire") {
        //     Async::NotReady => {
        //         // self.pending_accept = Some(pending);
        //         // return Err(mio::would_block())
        //         ()
        //     },
        //         Async::Ready(r) => {
        //             self.set_foo(r);
        //             ()
        //         },
        // }

        // match self.foo {
        //     Some(x) => {
        //         return Ok(futures::Async::Ready(self.x + x));
        //     },
        //     None =>  {
        //         self.task = Some(task::park());

        //         match self.rx.poll().expect("fallisce qua") {
        //             Async::NotReady => {
        //                 // self.pending_accept = Some(pending);
        //                 // return Err(mio::would_block())
        //                 ()
        //             },
        //             Async::Ready(r) => {
        //                 self.set_foo(r);
        //                 ()
        //             },
        //         };

        //         // self.rx.and_then(|x| {
        //         //     self.set_foo(x);
        //         //     Ok(())
        //         // });

        //         return Ok(futures::Async::NotReady)
        //     }
        // }
    }
}

fn main() {
    env_logger::init().unwrap();


    let mut eventloop = Core::new().unwrap();
    let handle = eventloop.handle();
    let (tx, rx) = oneshot::channel();

    let mut f = Sample {
        x: 20,
        handle: handle.clone(),
        task: None,
        foo: None,
        rx: rx
    };

    // let t = Timeout::new(Duration::new(5, 0), &handle).into_future().flatten();
    // let ft = t.and_then(move |_| {
    //     println!("Timed out");
    //     tx.complete(3);
    //     Ok(())
    // });
    // handle.spawn(ft.map_err(|_| ()));



    let addr = "127.0.0.1:9876".parse().unwrap();
    let tcp = TcpStream::connect(&addr, &handle);
    let client = tcp.and_then(|stream| {
        let (sink, mut stream) = stream.framed(LineCodec).split();
        let write_stdout = stream.poll().and_then(move |message| {
            // println!("{}", message);
            tx.complete(55);
            Ok(())
        });

        write_stdout

        // write_stdout.map(|_| ())
        //     .then(|_| Ok(()))
    });

    let client = client
        .map(|_| ())
        .map_err(|_| ());

    handle.spawn(client);


    let af = f.and_then(|x| {
        println!("{:?}", x);
        Ok(x)
    });

    let x = eventloop.run(af).unwrap();
    println!("{:?}", x);

    //******************************************

    // let mut eventloop = Core::new().unwrap();
    // let handle = eventloop.handle();

    // let (tx_intra, rx_intra) = mpsc::unbounded();

    // let mut f = Sample {
    //     x: 20,
    //     handle: handle.clone(),
    //     task: None,
    //     foo: None,
    //     rx: rx_intra,
    // };

    // let rf = Rc::new(RefCell::new(f));


    // let t = Timeout::new(Duration::new(2, 0), &handle).into_future().flatten();
    // let ft = t.and_then(move |_| {
    //     println!("Timed out");
    //     rf.borrow_mut().set_foo(2);

    //     let ff = rf.into_inner().and_then(|x| {
    //         Ok(x * 2)
    //     });
    //     ff
    //     // Ok(())
    // });
    // // handle.spawn(ft.map_err(|_| ()));



    // let (tx, rx) = mpsc::unbounded();
    // let address = "0.0.0.0:12345".parse().unwrap();
    // let listener = TcpListener::bind(&address, &handle).unwrap();
    // let connections = listener.incoming();
    // let handle_cloned = handle.clone();
    // let server = connections.for_each(move |(socket, _)| {
    //     let transport = socket.framed(LineCodec);
    //     let nonhocapitoperchedevoclonarlo = tx.clone();
    //     let tx_intra_cloned = tx_intra.clone();
    //     let process_connection = transport.for_each(move |line| {
    //         println!("{}", line);
    //         tx_intra_cloned.clone().send(true);
    //         nonhocapitoperchedevoclonarlo.clone().send(line)
    //             .map_err(|err| io::Error::new(ErrorKind::Other, err))
    //             .map(|_| ())
    //     });

    //     handle_cloned.clone().spawn(process_connection.map_err(|_| ()));

    //     Ok(())
    // });

    // handle.spawn(server.map_err(|_| ()));

    // let x = eventloop.run(af).unwrap();
    // println!("{:?}", x);
}
