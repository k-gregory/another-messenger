extern crate sdl2;
use std::sync::mpsc;

pub struct VoIpPlayCallback {
    rx: mpsc::Receiver<Vec<f32>>
}

impl VoIpPlayCallback{
    pub fn new(rx: mpsc::Receiver<Vec<f32>>) -> VoIpPlayCallback{
        VoIpPlayCallback{
            rx
        }
    }
}
impl sdl2::audio::AudioCallback for VoIpPlayCallback{
    type Channel = f32;

    fn callback(&mut self, r: &mut [Self::Channel]) {
        match self.rx.try_recv() {
            Ok(v) => {
                assert_eq!(v.len(), r.len());

                for i in 0..r.len() {
                    r[i] = v[i];
                }
            },
            Err(mpsc::TryRecvError::Disconnected) => {
                panic!("Disconnected audio")
            },
            Err(mpsc::TryRecvError::Empty) => {
                for i in 0..r.len(){
                    r[i] = 0.0;
                }
            }
        }
    }
}


pub struct VoIpCaptureCallback{
    tx: mpsc::Sender<Vec<f32>>
}

impl VoIpCaptureCallback{
    pub fn new(tx: mpsc::Sender<Vec<f32>>) -> VoIpCaptureCallback{
        VoIpCaptureCallback{
            tx
        }
    }
}
impl sdl2::audio::AudioCallback for VoIpCaptureCallback{
    type Channel = f32;

    fn callback(&mut self, c: &mut [Self::Channel]) {
        //println!("Capture callback");
        self.tx.send(c.to_vec()).unwrap()
    }
}