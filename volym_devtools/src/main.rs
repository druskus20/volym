use regex::Regex;
use serde_json::json;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

#[derive(serde::Serialize)]
struct Segment {
    index: u8,
    name: String,
    id: String,
    label_value: u8,
    importance: u8,
}

fn main() -> std::io::Result<()> {
    let collect = std::env::args().collect();
    let args: Vec<String> = collect;
    if args.len() != 4 {
        eprintln!(
            "Usage: {} <input.nrrd> <segments.json> <binary_data.bin>",
            args[0]
        );
        std::process::exit(1);
    }
    let input_path = &args[1];
    let json_output_path = &args[2];
    let binary_output_path = &args[3];

    read_segments_to_json(input_path, json_output_path)?;
    read_volume_data_to_file(input_path, binary_output_path)?;

    Ok(())
}

fn read_segments_to_json(
    input_path: &String,
    json_output_path: &String,
) -> Result<(), std::io::Error> {
    let file = File::open(input_path)?;
    let reader = BufReader::new(&file);
    let name_regex = Regex::new(r"Segment(\d+)_Name:=(.*)").unwrap();
    let id_regex = Regex::new(r"Segment(\d+)_ID:=(.*)").unwrap();
    let label_value_regex = Regex::new(r"Segment(\d+)_LabelValue:=(.*)").unwrap();
    let mut segment_names: std::collections::HashMap<u8, String> = std::collections::HashMap::new();
    let mut segment_ids: std::collections::HashMap<u8, String> = std::collections::HashMap::new();
    let mut segment_label_values: std::collections::HashMap<u8, String> =
        std::collections::HashMap::new();
    for line in reader.lines() {
        let line = line?;
        if let Some(captures) = name_regex.captures(&line) {
            let index = captures.get(1).unwrap().as_str().parse::<u8>().unwrap();
            let name = captures.get(2).unwrap().as_str().to_string();
            segment_names.insert(index, name);
        } else if let Some(captures) = id_regex.captures(&line) {
            let index = captures.get(1).unwrap().as_str().parse::<u8>().unwrap();
            let id = captures.get(2).unwrap().as_str().to_string();
            segment_ids.insert(index, id);
        } else if let Some(captures) = label_value_regex.captures(&line) {
            let index = captures.get(1).unwrap().as_str().parse::<u8>().unwrap();
            let label_value = captures.get(2).unwrap().as_str().parse::<u8>().unwrap();
            segment_label_values.insert(index, label_value.to_string());
        }
    }
    let segments: Vec<Segment> = segment_names
        .iter()
        .map(|(index, name)| Segment {
            index: *index,
            name: name.clone(),
            id: segment_ids.get(index).unwrap().clone(),
            label_value: segment_label_values
                .get(index)
                .unwrap()
                .parse::<u8>()
                .unwrap(),
            importance: 0,
        })
        .collect();
    let json_output = File::create(json_output_path)?;
    let mut json_writer = std::io::BufWriter::new(json_output);
    let json_data = json!(segments);
    json_writer.write_all(json_data.to_string().as_bytes())?;
    Ok(())
}

fn read_volume_data_to_file(
    input_path: &String,
    binary_output_path: &String,
) -> Result<(), std::io::Error> {
    let input_file = File::open(input_path)?;
    let lines = BufReader::new(&input_file).lines();
    let last_line = lines.last().unwrap().unwrap();
    let mut binary_output = File::create(binary_output_path)?;
    binary_output.write_all(&last_line.as_bytes())?;
    Ok(())
}
