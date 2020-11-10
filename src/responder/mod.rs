use std::{
    sync::mpsc,
    time,
    fs,
    cmp::Ordering,
};
use super::Packet;

pub fn output_delta(producer: mpsc::Receiver<(Packet, time::SystemTime)>, analyser: mpsc::Receiver<(Packet, time::SystemTime, u8)>) {
    let mut start_times = producer.try_iter().collect::<Vec<_>>();
    let mut analysis_times = analyser.try_iter().collect::<Vec<_>>();

    start_times.sort_by(|a, b| {
        let arr = a.0.0;
        let brr = b.0.0;

        let mut result = Ordering::Equal;
        let mut byte = 4;
        while (byte > 0) && (result == Ordering::Equal) {
            byte -= 1;
            result = arr[byte].cmp(&brr[byte]);
        }
        result
    });

    analysis_times.sort_by(|a, b| {
        let arr = a.0.0;
        let brr = b.0.0;

        let mut result = Ordering::Equal;
        let mut byte = 4;
        while (byte > 0) && (result == Ordering::Equal) {
            byte -= 1;
            result = arr[byte].cmp(&brr[byte]);
        }
        result
    });

    let mut data = String::new();

    for analysed in analysis_times.into_iter() {
        let (packet, end_time, path) = analysed;
        let id_bytes = packet.0;

        let id = ((id_bytes[0] as u32) & 0x000000ff)
            | (((id_bytes[1] as u32) <<  8) & 0x0000ff00)
            | (((id_bytes[2] as u32) << 16) & 0x00ff0000)
            | (((id_bytes[3] as u32) << 24) & 0xff000000);

        let original_index = start_times.binary_search_by(|probe| {
            let arr = probe.0.0;
    
            let mut result = Ordering::Equal;
            let mut byte = 4;
            while (byte > 0) && (result == Ordering::Equal) {
                byte -= 1;
                result = arr[byte].cmp(&id_bytes[byte]);
            }
            result
        }).unwrap();

        let value = end_time.duration_since(start_times[original_index].1).unwrap().as_micros();

        data = format!("{}{},{}\n", data,
            id,
            path_format(path, value),
        );
    }
    fs::write("output.csv", data).expect("Unable to write file");
}

fn path_format(path: u8, value: u128) -> String {
    let mut output = String::new();
    for i in 0..4 {
        if path == i {
            output = format!{"{}{}", output, value};
        }
        output = format!{"{},", output};
    }
    output
}