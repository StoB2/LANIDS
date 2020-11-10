mod pipeline;

use std::{
    collections::VecDeque,
    sync::mpsc,
    time,
};
use crate::Packet;

use super::{GPU_PARALLEL, CPU_PARALLEL};

pub struct Closing {
    inbox: mpsc::Receiver<(Packet, u8)>,
    outbox: mpsc::Sender<(Packet, time::SystemTime, u8)>,

    compute_pipeline: pipeline::ClosingPipeline,
}

impl Closing{
    pub fn new(gpu_compute_set: super::GPUComputeSet, inbox: mpsc::Receiver<(Packet, u8)>, outbox: mpsc::Sender<(Packet, time::SystemTime, u8)>) -> Self {        
        let compute_pipeline = pipeline::ClosingPipeline::new(gpu_compute_set);

        Self{
            inbox,
            outbox,
            compute_pipeline,
        }
    }

    pub fn run(self){
        use rayon::prelude::*;

        let mut connected = true;
        let mut load_queue = VecDeque::with_capacity(GPU_PARALLEL * 2);

        while connected || !load_queue.is_empty() {
            let mut incoming = true;
            while incoming {
                match self.inbox.try_recv() {
                    Ok(packet) => {load_queue.push_back(packet)},
                    Err(mpsc::TryRecvError::Empty) => {incoming = false;},
                    Err(mpsc::TryRecvError::Disconnected) => {
                        incoming = false;
                        connected = false;
                    },
                }
            }

            if load_queue.len() >= super::THRESHOLD {
                let mut gpu_workload = Vec::with_capacity(GPU_PARALLEL);
                while (gpu_workload.len() < GPU_PARALLEL) && !load_queue.is_empty() {
                    gpu_workload.push(load_queue.pop_front().unwrap());
                }
                self.compute_pipeline.scan(gpu_workload, self.outbox.clone());
            } else {
                let mut cpu_workload = Vec::with_capacity(CPU_PARALLEL);
                while (cpu_workload.len() < CPU_PARALLEL) && !load_queue.is_empty() {
                    cpu_workload.push(load_queue.pop_front().unwrap());
                }

                cpu_workload.into_par_iter().for_each_with(self.outbox.clone(), |s,p| {
                    if Self::cpu_closing(&p.0) {
                        s.send((p.0, time::SystemTime::now(), p.1)).unwrap();
                    }
                });
            }
        }
    }

    fn cpu_closing(packet: &Packet) -> bool {
        let mut intrusive = false;

        let payload = packet.0.to_vec();

        for chunk in (payload.len() / 4)..payload.len() {
            for bit in 0..8 {
                intrusive |= (payload[chunk] & crate::BIT_MASKS[bit]) > 0;
            }
        }

        intrusive
    }
}