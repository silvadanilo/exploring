extern crate futures;
extern crate tokio_core;
extern crate tokio_line;

use futures::{Async, AsyncSink, StartSend, Poll, Stream, Sink, IntoFuture};
use futures::future::{self, Future, Loop};
use futures::sync::oneshot;
use futures::sync::mpsc::{self, UnboundedSender};
use futures::stream::SplitSink;
use std::{io, str};
use tokio_core::io::{Io, Framed};
use tokio_core::net::{TcpStream};
use tokio_core::reactor::{Core, Handle};
use tokio_line::LineCodec;
use std::{thread, time};
use std::sync::Mutex;
use std::sync::Arc;
use std::sync::MutexGuard;
use std::ops::DerefMut;

type ConnToServerSink = SplitSink<Framed<TcpStream, LineCodec>>;

fn get_connection(handle: &Handle, tx: UnboundedSender<ConnToServerSink>) -> Box<Future<Item = (), Error = io::Error>> {
    let remote_addr = "127.0.0.1:9876".parse().unwrap();
    let tcp = TcpStream::connect(&remote_addr, handle);

    let handle_cloned = handle.clone();
    let client = tcp.and_then(move |stream| {
        let (sender, receiver) = stream.framed(LineCodec).split();

        let synkSink = tx.send(sender)
            .map(|_| ())
            .map_err(|_| ())
        ;
        handle_cloned.clone().spawn(synkSink);

        let reader = receiver.for_each(|message| {
            println!("{}", message);
            Ok(())
        });

        reader.and_then(|_| {
            println!("CLIENT DISCONNECTED");
            Ok(())
        })
    });

    let client = client.or_else(|_| {
        println!("connection refuse");
        thread::sleep(time::Duration::from_millis(100));
        // Ok(())
        Err(io::Error::new(io::ErrorKind::Other, "connection refuse"))
    });

    Box::new(client)
}


struct Buffer {
    tx: Option<SplitSink<Framed<TcpStream, LineCodec>>>,
    handle: Handle
}

impl Buffer {
    fn new(handle: Handle) -> Self {
        Buffer {
            tx: None,
            handle: handle,
        }
    }

    fn set_tx(&mut self, tx: SplitSink<Framed<TcpStream, LineCodec>>) {
        self.tx = Some(tx);
    }

    fn get_connection(&mut self) {

        let (ostx, osrx) = oneshot::channel();
        let get_sender = osrx.map(|sender: ConnToServerSink| -> Result<(), ()> {
            println!("sender ricevuto");
            self.set_tx(sender);
            Ok(())
        }).map(|_| ()).map_err(|_| ());
        self.handle.spawn(get_sender);

        let remote_addr = "127.0.0.1:9876".parse().unwrap();
        let tcp = TcpStream::connect(&remote_addr, &self.handle);

        let handle_cloned = self.handle.clone();
        let client = tcp.and_then(move |stream| {
            let (sender, receiver) = stream.framed(LineCodec).split();

            ostx.complete(sender);

            let reader = receiver.for_each(|message| {
                println!("{}", message);
                Ok(())
            });

            reader.and_then(|_| {
                println!("CLIENT DISCONNECTED");
                Ok(())
            })
        });

        let client = client.or_else(|_| {
            println!("connection refuse");
            thread::sleep(time::Duration::from_millis(100));
            // Ok(())
            Err(io::Error::new(io::ErrorKind::Other, "connection refuse"))
        }).map_err(|_| ());

        self.handle.spawn(client);
    }
}

impl Sink for Buffer {
    type SinkItem = String;
    type SinkError = ();

    fn start_send(&mut self, msg: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        println!("sink has received: {}", msg);

        match self.tx {
            None => {
                // get_connection(&handle, sinktx.clone())
                //     .map(|sender| -> Loop<(), ()> {
                //         Loop::Continue(())
                //     }).or_else(|_| -> Result<Loop<(), ()>, ()> {
                //         Ok(Loop::Continue(()))
                //     })
                self.get_connection();
                Ok(AsyncSink::NotReady(msg))
            }
            Some(ref mut tx) => {
                tx.start_send(msg);
                Ok(AsyncSink::Ready)
            }
        }
        // Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), ()> {
        match self.tx {
            None => {
                Ok(Async::NotReady)
                // get_connection(&handle, sinktx.clone())
                //     .map(|sender| -> Loop<(), ()> {
                //         Loop::Continue(())
                //     }).or_else(|_| -> Result<Loop<(), ()>, ()> {
                //         Ok(Loop::Continue(()))
                //     })
            }

            Some(ref mut tx) => {
                tx.poll_complete();
                Ok(Async::Ready(()))
            }
        }
    }
}

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let mut buffer = Buffer::new(handle.clone());

    let (buftx, bufrx) = mpsc::unbounded();

    let send_to_buftx = buftx
        .send("messaggio 1")
        .map(|_| ())
        .map_err(|_| ());
    handle.spawn(send_to_buftx);

    let f = bufrx
        .fold(buffer, |buffer, message| {
            buffer.send(message.to_string()).map_err(|_| ())
        })
        .map(|_| {
            println!("184");
            ()
        })
        .map_err(|_| {
            println!("188");
            ()
        });

    core.run(f).unwrap();
}

// fn amain() {
//     let mut core = Core::new().unwrap();
//     let handle = core.handle();
//     // let mut buffer = Buffer::new();

//     let arc_buffer = Arc::new(Mutex::new(Buffer::new()));
//     let arc_buffer_cloned = arc_buffer.clone();

//     let mut tx_set: Vec<SplitSink<Framed<TcpStream, LineCodec>>> = Vec::new();

//     let (sinktx, sinkrx) = mpsc::unbounded(); //change me in something bounded
//     let (buftx, bufrx) = mpsc::unbounded();
//     let send_to_buftx = buftx
//         .send("messaggio 1")
//         .map(|_| ())
//         .map_err(|_| ());
//     handle.spawn(send_to_buftx);

//     {
//         let addSinkToSet = sinkrx.for_each(move |sink| {
//             let mut buffer = arc_buffer_cloned.lock().expect("unable to lock");
//             buffer.set_tx(sink);
//             // tx_set.push(sink);
//             println!("sink pushato");
//             Ok(())
//         });

//         handle.spawn(addSinkToSet);
//     }

//     {
//         let arc_buffer_cloned = arc_buffer.clone();
//         let f = bufrx.for_each(move |msg| {
//             let mut buffer = arc_buffer_cloned.lock().expect("unable to lock");
//             // buffer.send(msg.to_string());
//             Ok(())
//         })
//         .map(|_| ())
//         .map_err(|_| ());
//         // let buffer = arc_buffer.lock().expect("Unable to lock output");
//         // let f = bufrx
//         //     .fold(buffer.deref_mut(), |buffer, message| {
//         //         buffer.send(message.to_string()).map_err(|_| ())
//         //     })
//         //     .map(|_| {
//         //         println!("184");
//         //         ()
//         //     })
//         //     .map_err(|_| {
//         //         println!("188");
//         //         ()
//         //     });

//         handle.spawn(f);
//     }


//     let client = future::loop_fn((), move |_| {
//         // let handle_cloned = handle.clone();
//         // let f = rx.and_then(|sender: SplitSink<Framed<TcpStream, LineCodec>>| {
//         //     Ok(Loop::Break(sender))
//         //     // tx_set.push(sender);
//         //     // Ok(())
//         // }).map_err(|_| ());
//         // handle.spawn(f);

//         // Run the get_connection function and loop again regardless of its result
//         get_connection(&handle, sinktx.clone())
//             .map(|sender| -> Loop<(), ()> {
//                 Loop::Continue(())
//             }).or_else(|_| -> Result<Loop<(), ()>, ()> {
//                 Ok(Loop::Continue(()))
//             })
//     });

//     // handle.spawn(x);

//     core.run(client).unwrap();
// }
