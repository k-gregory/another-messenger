extern crate sdl2;
extern crate mio;
extern crate bincode;
#[macro_use]
extern crate clap;

mod mpsc_evented;
mod voip_callbacks;

use mio::{Poll, Token, Ready, PollOpt, Events};
use std::error::Error;
use std::sync::mpsc;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use voip_callbacks::*;
use bincode::{serialize, deserialize, Infinite};

const DEFAULT_PORT: u16 = 4567;

const AUDIO_CAPTURED: Token = Token(0);
const PACKET_RECEIVED: Token = Token(1);

const EVENTS_CAPACITY: usize = 1024;



fn validate_port(s: String) -> Result<(), String> {
    s.parse::<u16>().map(|_| ()).map_err(|e| e.description().to_string())
}

fn prompt_port() -> u16 {
    let mut buff = String::new();
    loop {
        println!("Enter other program port");
        buff.clear();
        std::io::stdin().read_line(&mut buff).expect("Can't read line");
        let input = buff.trim();
        match input.parse::<u16>(){
            Ok(v) => return v,
            Err(e) => {
                println!("Can't read port number: {}", e.description());
            }
        }
    }
}

fn main(){

    //Parse command line
    let matches = clap_app!(myapp =>
        (version: "1.0.0-SNAPSHOT")
        (author: "Gregory K. <gregory_k@hotmail.com>")
        (about: "Research of VOIP implementation using Rust")
        (@arg NOCAPTURE: -s --nocapture "Do not capture the microphone audio")
        (@arg PORT: {validate_port} -p --port +takes_value "Selects port to listen")
        (@arg CLIENT: {validate_port} -c --client +takes_value "Client port number")
    ).get_matches();

    let port = matches
        .value_of("PORT")
        .map(|s| s.parse().unwrap())
        .unwrap_or(DEFAULT_PORT);

    let client_port = matches
        .value_of("CLIENT")
        .map(|s| s.parse().unwrap())
        .unwrap_or_else(|| prompt_port());

    let capture_desired = !matches.is_present("NOCAPTURE");

    let (playback_tx, playback_rx) = mpsc::channel();
    let (capture_tx, capture_rx) = mpsc::channel();
    let mut capture_rx_evented = mpsc_evented::EventedConsumer::new(capture_rx);

    //Setup audio playback and capture
    let sdl_context = sdl2::init().unwrap();
    let audio = sdl_context.audio().unwrap();
    let spec = sdl2::audio::AudioSpecDesired {
        channels: Some(1),
        freq: None,
        samples: Some(256)
    };
    let playback = audio.open_playback(None, &spec, |s|{
        println!("Playback: {:?}", s);
        VoIpPlayCallback::new(playback_rx)
    }).unwrap();

    let capture = if capture_desired {
        let capture = audio.open_capture(None, &spec, |s| {
            println!("Capture: {:?}", s);
            VoIpCaptureCallback::new(capture_tx)
        }).unwrap();
        capture.resume();
        Some(capture)
    } else { None };
    playback.resume();

    /*
    let mut events = Events::with_capacity(EVENTS_CAPACITY);
    let poll = Poll::new().unwrap();
    poll.register(&capture_rx_evented, INCOMING_AUDIO, Ready::readable(), PollOpt::edge()).unwrap();
    loop{
        poll.poll(&mut events, None).unwrap();
        for ev in events.iter(){
            match ev.token() {
                INCOMING_AUDIO => {
                    match capture_rx_evented.try_recv() {
                        Ok(v) => {
                            let forward = serialize(&v, Infinite).unwrap();
                            let backward: Vec<f32> = deserialize(&forward).unwrap();

                            playback_tx.send(backward).unwrap()
                        },
                        Err(mpsc::TryRecvError::Empty) => {},
                        Err(mpsc::TryRecvError::Disconnected) => panic!("Lol, disconnect")
                    }
                }
                _ => unreachable!()
            }
        }
    }
    */



    //Setup socket
    let socket = std::net::UdpSocket::bind(("0.0.0.0", port)).expect("Can't bind socket");
    let socket = mio::net::UdpSocket::from_socket(socket).unwrap();
    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), client_port);
    println!("You: {}, client: {}", port, client_port);

    //Run eventloop
    let mut socket_vec = vec![0; 550];

    let mut events = Events::with_capacity(EVENTS_CAPACITY);
    let poll = Poll::new().unwrap();
    poll.register(&socket, PACKET_RECEIVED, Ready::readable(), PollOpt::edge()).unwrap();
    poll.register(&capture_rx_evented, AUDIO_CAPTURED, Ready::readable(), PollOpt::edge()).unwrap();
    loop {
        poll.poll(&mut events, None).unwrap();
        for ev in events.iter() {
            match ev.token() {
                AUDIO_CAPTURED => {
                    loop {
                        match capture_rx_evented.try_recv() {
                            Ok(v) => {
                                let forward = serialize(&v, Infinite).unwrap();
                                socket.send_to(&forward, &socket_addr).unwrap();
                            },
                            Err(mpsc::TryRecvError::Empty) => {break},
                            Err(mpsc::TryRecvError::Disconnected) => panic!("Lol, disconnect")
                        };
                    }
                },
                PACKET_RECEIVED => {
                    println!("Packet received");
                    while let Ok((size,_)) = socket.recv_from(&mut socket_vec){
                        socket_vec.resize(size,0);
                        let data = deserialize(&socket_vec).unwrap();
                        playback_tx.send(data).unwrap();
                        socket_vec.resize(550,0);
                    }
                }
                _ => unreachable!()
            }
        }
    }

}