use std::{
    sync::mpsc,
    time,
};

use super::Packet;

pub fn consecutive256(analyser_send: mpsc::Sender<Packet>, responder_send: mpsc::Sender<(Packet, time::SystemTime)>) {
    for id in 0u32..=256u32 {
        let packet = if true {
            Packet::intrusive(id)
        } else {
            Packet::benign(id)
        };
        
        responder_send.send((packet, time::SystemTime::now())).unwrap();
        analyser_send.send(packet).unwrap();

        std::thread::sleep(time::Duration::from_micros(500));
    }
}