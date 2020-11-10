use std::{
    sync::mpsc,
    time,
    fs,
};
use super::Packet;

pub fn output_delta(producer: mpsc::Receiver<(Packet, time::SystemTime)>, analyser: mpsc::Receiver<(Packet, time::SystemTime)>) {
    let mut start_times = producer.try_iter().collect::<Vec<_>>();
    let analysis_times = analyser.try_iter().collect::<Vec<_>>();

    start_times.sort_by(|a, b| {a.0.0[0].cmp(&b.0.0[0])});

    let mut data = String::new();

    for analysed in analysis_times.into_iter() {
        let (packet, end_time) = analysed;
        let id = packet.0[0];

        let original_index = start_times.binary_search_by(|probe| {probe.0.0[0].cmp(&id)}).unwrap();

        data = format!("{}{}: {:?}\n", data, id, end_time.duration_since(start_times[original_index].1).unwrap() );
    }
    fs::write("output.txt", data).expect("Unable to write file");
}