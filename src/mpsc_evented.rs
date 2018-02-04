extern crate mio;

use mio::{Evented, Registration, Poll, Token, Ready, PollOpt};
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::sync::{Mutex, Arc, Condvar};
use std::thread;
use std::io;
use std::mem;

enum PipingState<T> {
    Waiting,
    Read(T),
    Disconnected
}

use self::PipingState::*;

struct Inner<T> {
    state: Mutex<PipingState<T>>,
    cvar: Condvar
}

pub struct EventedConsumer<T> where T: Send {
    registration: Registration,
    inner: Arc<Inner<T>>
}

impl<'a, T> EventedConsumer<T> where T:Send + 'static {
    pub fn new(rx: mpsc::Receiver<T>) -> EventedConsumer<T> {
        let inner1 = Arc::new(Inner {
            state: Mutex::new(Waiting),
            cvar: Condvar::new()
        });

        let (registration, set_readiness) = Registration::new2();

        let inner2 = inner1.clone();
        thread::spawn(move || {
            let inner = &*inner2;

            let mut exit = false;
            loop {
                if exit { break; }

                let new_value = match rx.recv() {
                    Err(mpsc::RecvError) => {
                        exit = true;
                        Disconnected
                    },
                    Ok(new_value) => {
                        Read(new_value)
                    }
                };


                let mut g = inner.state.lock().unwrap();
                while let Read(_) = *g {
                    g = inner.cvar.wait(g).unwrap();
                }
                *g = new_value;
                if !exit { set_readiness.set_readiness(Ready::readable()).unwrap(); }
            }
        });

        EventedConsumer {
            registration,
            inner: inner1
        }
    }

    pub fn try_recv(&mut self) -> Result<T, mpsc::TryRecvError> {
        let inner = &*self.inner;
        let cvar = &inner.cvar;
        let mut g = inner.state.lock().unwrap();
        let state = mem::replace(&mut *g, Waiting);

        match state {
            Waiting => Err(TryRecvError::Empty),
            Disconnected => {
                *g = Disconnected;
                Err(TryRecvError::Disconnected)
            },
            Read(r) => {
                cvar.notify_one();
                Ok(r)
            }
        }
    }
}

impl<T> Evented for EventedConsumer<T> where T: Send {
    fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt)
                -> io::Result<()>
    {
        self.registration.register(poll, token, interest, opts)
    }

    fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt)
                  -> io::Result<()>
    {
        self.registration.reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        self.registration.deregister(poll)
    }
}