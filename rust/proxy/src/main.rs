#[macro_use] extern crate log;
extern crate env_logger;

extern crate futures;
extern crate tokio_core;
extern crate tokio_line;
extern crate tokio_timer;

use futures::future::Future;
use futures::{Async, AsyncSink, Poll, StartSend, Stream, Sink};
// use futures::stream::{SplitSink};
use futures::sync::mpsc::{self, UnboundedSender};
use tokio_timer::*;
use tokio_core::io::{Io};
use tokio_core::net::{TcpStream, TcpStreamNew};
use tokio_core::reactor::{Core, Handle};
use tokio_line::LineCodec;
use std::{io, str};
use std::{thread, time};
use std::time::Duration;
use std::string::String;
use std::fmt;

enum RemoteConnectionState {
    NotConnected,
    Connecting(TcpStreamNew),
    Connected(UnboundedSender<String>),
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
        }
    }
}

struct StubbornSink {
    remote_addr: std::net::SocketAddr,
    status: RemoteConnectionState,
    handle: Handle,
}

impl StubbornSink {
    fn new(handle: Handle) -> Self {
        StubbornSink {
            remote_addr: "127.0.0.1:5550".parse().unwrap(),
            status: RemoteConnectionState::NotConnected,
            handle: handle,
        }
    }

    fn connection_attempt(&mut self) -> TcpStreamNew {
        TcpStream::connect(&self.remote_addr, &self.handle.clone())
    }

    // I have failed to pass &self here, because the `match` `Connecting` branch locks self. Try again!
    fn get_inner_sink(stream: TcpStream, handle: &Handle) -> UnboundedSender<String> {
        let (middleware_tx, middleware_rx) = mpsc::unbounded::<String>();

        let (sender, receiver) = stream.framed(LineCodec).split();

        let reader = receiver
            .for_each(|_message| {
                Ok(())
            })
            .and_then(|_| {
                info!("Connection with remote server is lost");
                Ok(())
            });

        let writer = middleware_rx
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "error kind returned should be the same of `sender` sink in `forward()`"))
            .forward(sender)
            .and_then(|(_rx, _tx)| Ok(()))
        ;

        let f = reader.select(writer)
            .map(|(res, _nf)| {
                res
            })
            .map_err(|(_err, _nf)| {
                ()
            })
            .and_then(move |_| {
                Ok(())
            });

        handle.spawn(f);

        middleware_tx
    }
}

impl Sink for StubbornSink {
    type SinkItem = String;
    type SinkError = io::Error;

    fn start_send(&mut self, msg: String) -> StartSend<String, io::Error> {
        debug!("current status: {}", self.status);

        loop {
            let next_status = match self.status {
                RemoteConnectionState::Connected(ref split_sink) => {
                    match split_sink.send(msg.clone()) {
                        Ok(_) => return Ok(AsyncSink::Ready),
                        Err(_) => Some(RemoteConnectionState::NotConnected),
                    }
                }
                RemoteConnectionState::Connecting(ref mut future) => {
                    match future.poll() {
                        Err(_) => {
                            thread::sleep(time::Duration::from_millis(50)); //TODO:! make millis configurable
                            Some(RemoteConnectionState::NotConnected)
                        }
                        Ok(Async::NotReady) => {
                            return Ok(AsyncSink::NotReady(msg));
                        }
                        Ok(Async::Ready(stream)) => {
                            let middleware_tx = StubbornSink::get_inner_sink(stream, &self.handle);
                            Some(RemoteConnectionState::Connected(middleware_tx))
                        }
                    }
                }
                RemoteConnectionState::NotConnected => {
                    Some(RemoteConnectionState::Connecting(self.connection_attempt()))
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

    let timer = Timer::default();
    let wakeups = timer.interval(Duration::new(0, 150000000));
    let mut i = 0;
    let background_tasks = wakeups.for_each(move |_| {
        debug!("Interval");
        i = i + 1;
        buftx.clone()
            // .send(format!("Messagio {}", i).to_string())
            .send(format!(r#"{{"@timestamp":"2017-03-24T09:16:42.636040+01:00","@source":"dev-all-onebiptrusty cli","@fields":{{"channel":"integrationtest-client","level":100,"extra_level_name":"DEBUG","extra_uname":"dev-all-onebiptrusty","extra_sapi":"cli","extra_process_id":17954}},"@message":"Message {}"}}"#, i).to_string())
            .map(|_| ())
            .map_err(|_| TimerError::NoCapacity)
    });

    // let background_tasks = buftx.clone()
    //     .send(format!("Messagio {}", i).to_string())
    //     .map(|_| ())
    //     .map_err(|_| ());

    handle.spawn(background_tasks.map(|_| ()).map_err(|_| ()));
}
