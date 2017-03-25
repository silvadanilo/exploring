#[macro_use] extern crate log;
extern crate env_logger;

extern crate futures;
extern crate tokio_core;
extern crate tokio_line;
extern crate tokio_timer;

use futures::future::Future;
use futures::{Async, AsyncSink, Poll, StartSend, Stream, Sink, sink};
use futures::stream::{SplitSink};
use futures::sync::mpsc::{self, UnboundedSender};
use tokio_timer::*;
use tokio_core::io::{Io, Framed};
use tokio_core::net::{TcpStream, TcpStreamNew};
use tokio_core::reactor::{Core, Handle};
use tokio_line::LineCodec;
use std::{io, str};
use std::{thread, time};
use std::time::Duration;
use std::string::String;
use std::fmt;
use std::mem;

enum RemoteConnectionState {
    NotConnected,
    Connecting(TcpStreamNew),
    Connected(UnboundedSender<String>),
    // Connected(SplitSink<Framed<TcpStream, LineCodec>>),
    // Sending,
}

impl fmt::Display for RemoteConnectionState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RemoteConnectionState::NotConnected => {
                write!(f, "NotConnected")
            }
            RemoteConnectionState::Connecting(_) => {
                write!(f, "Connecting")
            }
            RemoteConnectionState::Connected(_) => {
                write!(f, "Connected")
            }
            // RemoteConnectionState::Sending => {
            //     write!(f, "Sending")
            // }
        }
    }
}

struct StubbornSink {
    remote_addr: std::net::SocketAddr,
    status: RemoteConnectionState,
    handle: Handle,
    stream: Option<SplitSink<Framed<TcpStream, LineCodec>>>,
    // sending: Option<sink::Send<SplitSink<Framed<TcpStream, LineCodec>>>>,
}

impl StubbornSink {
    fn new(handle: Handle) -> Self {
        StubbornSink {
            remote_addr: "127.0.0.1:9876".parse().unwrap(),
            status: RemoteConnectionState::NotConnected,
            handle: handle,
            stream: None,
            // sending: None,
        }
    }

    fn try_to_connect(&mut self) -> TcpStreamNew {
        TcpStream::connect(&self.remote_addr, &self.handle.clone())
    }
}

impl Sink for StubbornSink {
    type SinkItem = String;
    type SinkError = io::Error;

    fn start_send(&mut self, msg: String) -> StartSend<String, io::Error> {
        debug!("current status: {}", self.status);

        loop {
            let next_status = match self.status {
                // RemoteConnectionState::Sending => {
                //     if let Some(ref mut sending_future) = self.sending {
                //         match sending_future.poll() {
                //             Err(_) => {
                //                 debug!("Error in sending msg `{}`", msg.clone());
                //                 Some(RemoteConnectionState::NotConnected)
                //             },
                //             Ok(Async::NotReady) => {
                //                 return Ok(AsyncSink::NotReady(msg));
                //             },
                //             Ok(Async::Ready(split_sink)) => {
                //                 // self.stream = Some(split_sink);
                //                 self.status = RemoteConnectionState::Connected(split_sink);
                //                 return Ok(AsyncSink::Ready);
                //             }
                //         }
                //     } else {
                //         error!("Should never happen");
                //         Some(RemoteConnectionState::NotConnected)
                //     }
                // },
                RemoteConnectionState::Connected(ref split_sink) => {
                    // self.sending = Some(split_sink.send(msg.clone()));
                    // Some(RemoteConnectionState::Sending)

                    // if let Some(sender) = self.stream.take() {
                    //     // let (sender, receiver) = stream.framed(LineCodec).split();
                    //     self.sending = Some(sender.send(msg.clone()));
                    //     Some(RemoteConnectionState::Sending)
                    //     // self.stream = Some(stream);
                    // } else{
                    //     Some(RemoteConnectionState::NotConnected)
                    // }


                    // if let Some(stream) = self.stream {
                    // if stream {
                        // let (sender, receiver) = stream.framed(LineCodec).split();
                        // let sending_future = sender.send(msg.clone());
                        // Some(RemoteConnectionState::Sending(sending_future))
                    // } else {
                    //     Some(RemoteConnectionState::NotConnected)
                    // }

                    match split_sink.send(msg.clone()) {
                        Ok(_) => {
                            return Ok(AsyncSink::Ready);
                        }
                        Err(_) => Some(RemoteConnectionState::NotConnected),
                    }
                }
                RemoteConnectionState::NotConnected => {
                    Some(RemoteConnectionState::Connecting(self.try_to_connect()))
                }
                RemoteConnectionState::Connecting(ref mut future) => {
                    match future.poll() {
                        Err(_) => {
                            thread::sleep(time::Duration::from_millis(50));
                            Some(RemoteConnectionState::NotConnected)
                        }
                        Ok(Async::NotReady) => {
                            return Ok(AsyncSink::NotReady(msg));
                        }
                        Ok(Async::Ready(stream)) => {
                            println!("1");
                            let (remote_tmp_tx, remote_tmp_rx) = mpsc::unbounded::<String>();
                            let (sender, receiver) = stream.framed(LineCodec).split();

                            // // self.stream = Some(sender);
                            // Some(RemoteConnectionState::Connected(sender))



                            let reader = receiver
                                .for_each(|message| {
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


                            let f = reader.select(writer)
                                .map(|(res, _nf)| {
                                    res
                                })
                                .map_err(|(err, _nf)| {
                                    // err
                                    ()
                                })
                                .and_then(move |_| {
                                    // buffer.borrow_mut().remove();
                                    Ok(())
                                });

                            self.handle.spawn(f);

                            println!("2");
                            Some(RemoteConnectionState::Connected(remote_tmp_tx))
                        }
                    }
                }
            };

            match next_status {
                Some(s) => self.status = s,
                None => {}
            }
        }
    }

    fn poll_complete(&mut self) -> Poll<(), io::Error> {
        debug!("my poll_complete");
        Ok(Async::Ready(()))
    }
}

fn main() {
    env_logger::init().unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let (buftx, bufrx) = mpsc::unbounded();

    simulated_messaging_receiving_from_clients(buftx.clone(), &handle.clone());

    let stubborn_sink = StubbornSink::new(handle.clone());
    let f =
        bufrx.fold(stubborn_sink,
                   |stubborn_sink, message| stubborn_sink.send(message).map_err(|_| ()));

    core.run(f).unwrap();
}

fn simulated_messaging_receiving_from_clients(buftx: UnboundedSender<String>,
                                              handle: &Handle)
                                              -> () {

    let handle_cloned = handle.clone();
    let timer = Timer::default();
    let wakeups = timer.interval(Duration::new(0, 150000000));
    let mut i = 0;
    let background_tasks = wakeups.for_each(move |_| {
        debug!("Interval");
        i = i + 1;
        buftx.clone()
            .send(format!("Messagio {}", i).to_string())
            .map(|_| ())
            .map_err(|_| TimerError::NoCapacity)
    });

    // let background_tasks = buftx.clone()
    //     .send(format!("Messagio {}", i).to_string())
    //     .map(|_| ())
    //     .map_err(|_| ());

    handle.spawn(background_tasks.map(|_| ()).map_err(|_| ()));
}

