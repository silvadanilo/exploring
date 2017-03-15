extern crate futures;
extern crate tokio_core;
extern crate tokio_line;

use futures::future::{self, Future, Loop, IntoFuture};
use futures::{Async, AsyncSink, Poll, StartSend, Stream, Sink};
use futures::sync::mpsc::{self, UnboundedSender, UnboundedReceiver};
use tokio_core::io::{Io};
use tokio_core::net::{TcpStream};
use tokio_core::reactor::{Core, Handle, Timeout};
use tokio_line::LineCodec;
use std::{io, str};
use std::{thread, time};
use std::time::Duration;
use std::rc::Rc;
use std::cell::RefCell;

struct Buffer {
    connection: Option<UnboundedSender<String>>,
    buftx: UnboundedSender<String>,
    handle: Handle,
    // buffer: Vec<String>,
}

impl Buffer {
    fn new(buftx: UnboundedSender<String>, handle: Handle) -> Self {
        Buffer {
            connection: None,
            buftx: buftx,
            handle: handle,
            // buffer: vec![],
        }
    }

    fn add(&mut self, tx: UnboundedSender<String>) {
        self.connection = Some(tx);
    }

    fn remove(&mut self) {
        self.connection = None;
    }

    // fn send(&self, message: String) -> Box<Future<Item = (), Error = ()>> {
    fn send(&self, message: String) -> Box<Result<(), ()>> {
        let f = match self.connection {
            Some(ref tx) => {
                tx.send(message) //UnboundedSender::send() returns a Result<(), SendError<T>>
                    .map(|_| ())
                    .map_err(|error| {
                        //self.remove(); //FIXME:! how to call remove?!?
                        self.push_back(error.into_inner());
                        ()
                    })
            },
            None => {
                //TODO:! message should not be lost but should not push back as is here
                self.push_back(message);

                println!("CONNECTION NOT FOUND");
                Err(())
            }
        };

        return Box::new(f);
    }

    fn push_back(&self, message: String) {
        let buftx_cloned = self.buftx.clone();
        let sent = buftx_cloned.send(message);
        self.handle.spawn(sent
            .map(|_| ())
            .map_err(|_| ())
        );
    }
}

// impl Sink for Buffer {
//     type SinkItem = String;
//     type SinkError = ();

//     fn start_send(&mut self, msg: String) -> StartSend<String, ()> {
//         println!("start send");
//         self.buffer.push(msg);
//         Ok(AsyncSink::Ready)
//     }

//     fn poll_complete(&mut self) -> Poll<(), ()> {
//         println!("poll complete");
//         Ok(Async::Ready(()))
//     }
// }


fn send_data_to_remote_server<'a>(handle: &Handle, buffer: Rc<RefCell<Buffer>>) -> Box<Future<Item = (), Error = io::Error>> {
    let remote_addr = "127.0.0.1:9876".parse().unwrap();
    let tcp = TcpStream::connect(&remote_addr, handle);

    let client = tcp.and_then(move |stream| {
        let (remote_tmp_tx, remote_tmp_rx) = mpsc::unbounded::<String>();
        buffer.borrow_mut().add(remote_tmp_tx);

        let (sender, receiver) = stream.framed(LineCodec).split();
        let reader = receiver
            .for_each(|_message| {
                // println!("{}", message);
                Ok(())
            })
            .and_then(|_| {
                println!("CLIENT DISCONNECTED");
                Ok(())
            });

        let writer = remote_tmp_rx
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "error kind returned should be the same of `sender` sink in `forward()`"))
            .forward(sender)
            .and_then(|(_rx, _tx)| Ok(()))
        ;

        reader.select(writer)
            .map(|(res, _nf)| {
                res
            })
            .map_err(|(err, _nf)| {
                err
            })
            .and_then(move |_| {
                buffer.borrow_mut().remove();
                Ok(())
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
    let buffer = Rc::new(RefCell::new(Buffer::new(
        buftx.clone(),
        handle.clone()
    )));

    simulated_messaging_receiving_from_clients(buftx.clone(), &handle.clone());

    let buffer_inner = buffer.clone();
    let f = bufrx.for_each(move |message| {
        buffer_inner.borrow().send(message.clone());
        Ok(())
    }).map_err(|_| ());

    handle.spawn(f);

    let client = future::loop_fn((), |_| {
        send_data_to_remote_server(&handle, buffer.clone())
            .map(|_| -> Loop<(), ()> {
                Loop::Continue(())
            })
            .or_else(|_| -> Result<Loop<(), ()>, ()> {
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
