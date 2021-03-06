use std::{
    sync::mpsc,
    time,
};

use super::Packet;

pub fn alternating25565(analyser_send: mpsc::Sender<Packet>, responder_send: mpsc::Sender<(Packet, time::SystemTime)>) {
    for id in 0u32..=25565u32 {    
        let packet = if (id % 2) == 0 {
            Packet::intrusive(id)
        } else {
            Packet::benign(id)
        };
        
        responder_send.send((packet, time::SystemTime::now())).unwrap();
        analyser_send.send(packet).unwrap();

        std::thread::sleep(time::Duration::from_nanos(50));
    }
}

pub fn alternating1024_low_load(analyser_send: mpsc::Sender<Packet>, responder_send: mpsc::Sender<(Packet, time::SystemTime)>) {
    for id in 0u32..=1024u32 {    
        let packet = if (id % 2) == 0 {
            Packet::intrusive(id)
        } else {
            Packet::benign(id)
        };
        
        responder_send.send((packet, time::SystemTime::now())).unwrap();
        analyser_send.send(packet).unwrap();

        std::thread::sleep(time::Duration::from_micros(2000));
    }
}

pub fn ramp_mid(analyser_send: mpsc::Sender<Packet>, responder_send: mpsc::Sender<(Packet, time::SystemTime)>) {
    for id in 0u32..=2048u32 {    
        let packet = if (id % 2) == 0 {
            Packet::intrusive(id)
        } else {
            Packet::benign(id)
        };
        
        responder_send.send((packet, time::SystemTime::now())).unwrap();
        analyser_send.send(packet).unwrap();

        if (id > 1024) && (id < 1536) {
            std::thread::sleep(time::Duration::from_micros(500));
        } else {
            std::thread::sleep(time::Duration::from_micros(2000));
        }

    }
}

pub fn samAndAlex(analyser_send: mpsc::Sender<Packet>, responder_send: mpsc::Sender<(Packet, time::SystemTime)>) {
    for id in 0u32..=25565u32 /*quantity*/  {
        // packet creation
        let packet = if (id % 2) == 0 {
            Packet::intrusive(id)
        } else {
            Packet::benign(id)
        };
        
        responder_send.send((packet, time::SystemTime::now())).unwrap();
        analyser_send.send(packet).unwrap();

        std::thread::sleep(time::Duration::from_nanos(1_000));
    }
}







pub fn alternating8192(analyser_send: mpsc::Sender<Packet>, responder_send: mpsc::Sender<(Packet, time::SystemTime)>) {
    for id in 0u32..=8192u32 {
        let packet = if (id % 2) == 0 {
            Packet::intrusive(id)
        } else {
            Packet::benign(id)
        };
        
        responder_send.send((packet, time::SystemTime::now())).unwrap();
        analyser_send.send(packet).unwrap();

        std::thread::sleep(time::Duration::from_nanos(1_000));
    }
}

pub fn alternating8192_low(analyser_send: mpsc::Sender<Packet>, responder_send: mpsc::Sender<(Packet, time::SystemTime)>) {
    for id in 0u32..=8192u32 {
        let packet = if (id % 2) == 0 {
            Packet::intrusive(id)
        } else {
            Packet::benign(id)
        };
        
        responder_send.send((packet, time::SystemTime::now())).unwrap();
        analyser_send.send(packet).unwrap();

        std::thread::sleep(time::Duration::from_nanos(100_000));
    }
}

pub fn time_wack(analyser_send: mpsc::Sender<Packet>, responder_send: mpsc::Sender<(Packet, time::SystemTime)>) {
    for id in 0u32..=8192u32 {    
        let packet = if ((id/32) % 2) == 0 {
            Packet::intrusive(id)
        } else if (id % 4) == 0 {
            Packet::suspicious(id)
        } else {
            Packet::benign(id)
        };

        responder_send.send((packet, time::SystemTime::now())).unwrap();
        analyser_send.send(packet).unwrap();

        if ((id/32) % 2) == 1 {
            std::thread::sleep(time::Duration::from_nanos(100_000));
        } else {
            std::thread::sleep(time::Duration::from_nanos(1_000));
        }
    }
}