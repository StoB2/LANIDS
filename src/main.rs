mod packet_producer;
mod packet_analyser;
mod responder;

use std::sync::mpsc;

pub const BIT_MASKS: [u8; 8] = [
    0b1000_0000,
    0b0100_0000,
    0b0010_0000,
    0b0001_0000,

    0b0000_1000,
    0b0000_0100,
    0b0000_0010,
    0b0000_0001,
];

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Packet([u8; Self::BYTE_COUNT]);

unsafe impl bytemuck::Pod for Packet {}
unsafe impl bytemuck::Zeroable for Packet {}

impl Packet{
    pub const BYTE_COUNT: usize = 4 * 1024;

    pub fn benign(id: u32) -> Self {
        let mut packet = Packet([0;Packet::BYTE_COUNT]);
        packet.0[0] = ((id >> 24) & 0xff) as u8;
        packet.0[1] = ((id >> 16) & 0xff) as u8;
        packet.0[2] = ((id >> 8) & 0xff) as u8;
        packet.0[3] = (id & 0xff) as u8;

        packet
    }

    pub fn suspicious(id: u32) -> Self {
        let mut packet = Packet([0;Packet::BYTE_COUNT]);
        packet.0[0] = ((id >> 24) & 0xff) as u8;
        packet.0[1] = ((id >> 16) & 0xff) as u8;
        packet.0[2] = ((id >> 8) & 0xff) as u8;
        packet.0[3] = (id & 0xff) as u8;

        packet.0[4] = 0b0000_0001;

        packet
    }

    pub fn intrusive(id: u32) -> Self {
        let mut packet = Packet([0;Packet::BYTE_COUNT]);
        packet.0[0] = ((id >> 24) & 0xff) as u8;
        packet.0[1] = ((id >> 16) & 0xff) as u8;
        packet.0[2] = ((id >> 8) & 0xff) as u8;
        packet.0[3] = (id & 0xff) as u8;

        packet.0[4] = 0b0000_0001;
        packet.0[Packet::BYTE_COUNT - 1] = 0b1000_0000;

        packet
    }
}

fn main() {
    let (analyser_tx, analyser_rx) = mpsc::channel();
    let (responder_raw_tx, responder_raw_rx) = mpsc::channel();
    let (responder_filter_tx, responder_filter_rx) = mpsc::channel();

    let analysis_threads = packet_analyser::start_analysers(analyser_rx, responder_filter_tx);

    std::thread::sleep(std::time::Duration::from_millis(10));
    
    packet_producer::consecutive256(analyser_tx, responder_raw_tx);

    for thread in analysis_threads.into_iter() {
        thread.join().unwrap();
    }

    responder::output_delta(responder_raw_rx, responder_filter_rx);
}