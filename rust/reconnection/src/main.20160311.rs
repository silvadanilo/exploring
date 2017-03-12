extern crate futures;
extern crate tokio_core;
extern crate tokio_line;

use futures::future::{self, Future, Loop, FutureResult, IntoFuture, ok};
use futures::{Sink, Stream};
use futures::sync::mpsc::{self, UnboundedSender, UnboundedReceiver};
use futures::stream::{Forward, SplitSink};
use std::{io, str};
use tokio_core::io::{Io};
use tokio_core::net::{TcpStream};
use tokio_core::reactor::{Core, Handle, Timeout};
use tokio_line::LineCodec;
use std::{thread, time};
use std::time::Duration;

fn get_test(handle: &Handle, bufrx: UnboundedReceiver<String>) -> Box<Future<Item = UnboundedReceiver<String>, Error = io::Error>> {
    let remote_addr = "127.0.0.1:9876".parse().unwrap();
    let tcp = TcpStream::connect(&remote_addr, handle);

    let client = tcp.and_then(|stream| {
        let (sender, receiver) = stream.framed(LineCodec).split();

        let bufrx = bufrx.map_err(|_| io::Error::new(io::ErrorKind::Other, "boh!"));

        let sync_with_server = bufrx
            .forward(sender)
            .and_then(|(bufrx2, sender)| {
                bufrx = bufrx2;
                Ok(())
            });

        // sync_with_server

        let reader = receiver.for_each(|message| {
            println!("{}", message);
            Ok(())
        })
        .map(|_| ())
        .map_err(|_| {
            io::Error::new(io::ErrorKind::Other, "boh!")
        });

        // let (_, fakerx) = mpsc::unbounded();
        // reader.select(sync_with_server.map_err(|_| bufrx))
        //     .map(|_| {
        //         // fakerx
        //         ()
        //     })
        //     .map_err(|_| {
        //         // fakerx
        //         ()
        //     })
        // reader.and_then(|_| {
        //     println!("CLIENT DISCONNECTED");
        //     Ok(bufrx)
        // })

        // let (buftx, bufrx) = mpsc::unbounded();
        // Ok(bufrx)

        reader
    });

    Box::new(client)
}

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let (buftx, bufrx) = mpsc::unbounded();

    simulated_messaging_receiving_from_clients(buftx.clone(), &handle.clone());

    // let client =get_test(&handle, bufrx)
    //     .and_then(|bufrx3| {
    //         println!("{}", "tutto finito");
    //         Ok(())
    //     });


    let client = future::loop_fn(bufrx, |bufrx2| {
        // Run the get_connection function and loop again regardless of its result
        get_test(&handle, bufrx2)
            .and_then(|bufrx3| -> Result<Loop<UnboundedReceiver<String>, UnboundedReceiver<String>>, io::Error> {
                println!("AND THEN LOOP");
                Ok(Loop::Continue(bufrx2))
            })
            // .or_else(|bufrx_test| {
            //     println!("OR ELSE LOOP");
            //     Ok(Loop::Continue(bufrx2))
            // })
    });


    core.run(client);
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

// fn get_connection(handle: &Handle, bufrx: UnboundedReceiver<String>) -> Box<Future<Item = UnboundedReceiver<String>, Error = io::Error>> {
// // fn get_connection(handle: &Handle, bufrx: UnboundedReceiver<String>) -> Box<Future<Item = UnboundedReceiver<String>, Error = io::Error>> {
// // fn get_connection(handle: &Handle, bufrx: UnboundedReceiver<String>) -> FutureResult<(UnboundedReceiver<String>, u8), io::Error> {
//     let remote_addr = "127.0.0.1:9876".parse().unwrap();
//     let tcp = TcpStream::connect(&remote_addr, handle);

//     let client = tcp.and_then(|stream| {
//     // let client = tcp.and_then(|stream| -> Result<UnboundedReceiver<String>, io::Error> {
//         let (sender, receiver) = stream.framed(LineCodec).split();

//         let f = bufrx
//             .fold(sender, |sender, message| {
//                 sender.send(message.to_string()).map_err(|_| ())
//             })
//             .map(|_| {
//                 // bufrx
//                 ()
//             });

//         // let f = f.map_err(|_| io::Error::new(io::ErrorKind::Other, "connection refuse"));

//         handle.spawn(f);

//         Ok(())

//         // let reader = receiver.for_each(|message| {
//         //     println!("{}", message);
//         //     Ok(())
//         // });

//         // reader.and_then(|_| {
//         //     println!("CLIENT DISCONNECTED");
//         //     Ok(bufrx)
//         // })
//     });

//     // let client = client
//     //     .and_then(|bufrx| {
//     //         Ok(bufrx)
//     //     })
//     //     .or_else(|e| -> Result<(), UnboundedReceiver<String>> {
//     //         Ok(())
//     //         // Ok(bufrx)
//     //         // Ok(())
//     //     });

//     // let client = client
//     //     .and_then(|bufrx| {
//     //         Ok(bufrx)
//     //     })
//     //     .or_else(|bufrx| {
//     //         println!("connection refuse");
//     //         thread::sleep(time::Duration::from_millis(100));
//     //         Ok(bufrx)
//     //         // Err(io::Error::new(io::ErrorKind::Other, "connection refuse"))
//     //     });

//     Box::new(client)
//     // Box::new(future::ok(bufrx))
//     // ok((bufrx, 1))
// }

// fn main() {
//     let mut core = Core::new().unwrap();
//     let handle = core.handle();
//     let (buftx, bufrx) = mpsc::unbounded();

//     let client = future::loop_fn(bufrx, |bufrx2| {
//         // Run the get_connection function and loop again regardless of its result
//         get_connection(&handle, bufrx2)
//             .and_then(|bufrx3| -> Result<Loop<UnboundedReceiver<String>, UnboundedReceiver<String>>, io::Error> {
//                 let (_, bufrx_test) = mpsc::unbounded();
//                 Ok(Loop::Continue(bufrx_test))
//             })
//     });

//     core.run(client);
// }
