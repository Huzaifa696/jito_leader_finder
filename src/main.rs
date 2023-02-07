use plotters::prelude::*;
use solana_client::rpc_client::RpcClient;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::process::Command;

const SOLANA_COMMAND: &str = "solana";
const LEADER_SCHEDULE_SUBCOMMAND: &str = "leader-schedule";
const OUTPUT_FILE: &str = "output.txt";

fn main() {
    // checking cli arguments
    let args: Vec<String> = env::args().collect();
    // println!("{:?}", args.len());
    // println!("{:?}", args.get(0));
    if args.len() != 5 {
        println!("ERROR - command line argumets error!");
        println!("Usage: cargo run <block-engine> <path-to-id> <rpc-node> <access-token-for-rpc>");
        return;
    }
    let block_engine = &args[1][..];
    let id_path = &args[2][..];
    let rpc_node = &args[3][..];
    let access_token = &args[4][..];

    // println!("{:?}", args);

    // get solana leader schedule and write to a file
    println!("Getting leader schedule from RPC node.");
    let output = Command::new(SOLANA_COMMAND)
        .arg(LEADER_SCHEDULE_SUBCOMMAND)
        .output()
        .expect("failed to execute process");
    let mut file = File::create(OUTPUT_FILE).expect("Unable to create file");
    file.write_all(&output.stdout)
        .expect("Unable to write data");

    // read leader schedule from file to a data structure
    println!("Parsing leader schedule into a hashmap.");
    let file = File::open(OUTPUT_FILE).expect("Unable to open file");
    let reader = BufReader::new(file);
    let mut map = HashMap::new();
    for line in reader.lines() {
        let line = line.expect("Unable to read line");
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let key = parts[0].trim().to_owned().parse::<u64>().unwrap();
            let value = parts[1].trim().to_owned();
            map.insert(key, value);
        }
    }

    // get jito leader slots
    println!("Requesting Jito`s leader schedule from jito CLI tool");
    let username = env::var("USER").unwrap();
    let script_path = format!("/home/{}/jito-searcher-cli", username);
    let full_rpc_node = &format!("{}/?access-token={}", rpc_node, access_token)[..];
    let output = Command::new(script_path)
        .args(&[
            "--block-engine-url",
            block_engine,
            "--keypair-path",
            id_path,
            "connected-leaders-info",
            "--rpc-url",
            full_rpc_node,
        ])
        .output()
        .expect("Failed to execute command");
    let stdout = output.stdout;
    let reader = BufReader::new(&stdout[..]);
    let jito_leaders: Vec<String> = reader.lines().map(|line| line.unwrap()).collect();

    // find jito leaders in the leader scheduler
    let mut entries: HashMap<u64, String> = HashMap::new();
    for (key, value) in &map {
        for leader in &jito_leaders {
            if value == leader {
                entries.insert(*key, value.clone());
            }
        }
    }

    // get current slot
    println!("Requesting current slot");
    let url = "https://api.mainnet-beta.solana.com".to_string();
    let client = RpcClient::new(url);
    let slot = client.get_slot().unwrap();

    // find future jito leaders
    let mut future_leaders: Vec<u64> = Vec::new();
    for (key, _) in &entries {
        match key.checked_sub(slot) {
            Some(result) => {
                if result > 150 {
                    future_leaders.push(*key);
                }
            }
            None => (),
        }
    }
    // println!("closet Jito slot: {}", future_leaders.iter().min().unwrap());

    // Convert the HashMap to a Vec of key-value pairs
    let mut sorted = map.keys().cloned().collect::<Vec<_>>();
    sorted.sort();

    let x_range = map.len() as u32; // no of slots
    let min_y_axis = u32::try_from(*map.keys().min().unwrap()).unwrap();
    let max_y_axis = u32::try_from(*map.keys().max().unwrap()).unwrap();
    // let mut jito_leader_points: Vec<(u32, u32)> = Vec::new();
    // for (i, x) in sorted.iter().enumerate() {
    //     if entries.contains_key(x) {
    //         jito_leader_points.push((i as u32, *x as u32));
    //     }
    // }
    let mut jito_leaders_concentration = Vec::new();
    let batch_size = (map.len() / 100) as u32;
    let half_batch = (batch_size / 2) as u32;
    let mut counter = 0;
    let mut sum = 0;
    let mut fraction: f64;
    for (i, x) in sorted.iter().enumerate() {
        counter = counter + 1;
        if entries.contains_key(x) {
            sum = sum + 1;
        }
        if counter == batch_size || counter == map.len() as u32 {
            fraction = sum as f64 / counter as f64;
            counter = 0;
            sum = 0;
            jito_leaders_concentration.push((
                i as u32 - half_batch as u32,
                *x as u32 - half_batch,
                fraction,
            ));
        }
    }

    let root_drawing_area = SVGBackend::new("bubble.png", (1600, 800)).into_drawing_area();
    root_drawing_area.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root_drawing_area)
        .caption("Jito Slot Distribution Visualizer", ("Arial", 30))
        .set_label_area_size(LabelAreaPosition::Left, 100)
        .set_label_area_size(LabelAreaPosition::Bottom, 100)
        .build_cartesian_2d(0..x_range, min_y_axis..max_y_axis)
        .unwrap();
    chart.configure_mesh().draw().unwrap();
    chart
        .draw_series(
            jito_leaders_concentration
                .iter()
                .map(|point| Circle::new((point.0, point.1), point.2 * 500 as f64, RED.filled())),
        )
        .unwrap();
    chart
        .draw_series(
            LineSeries::new((0..x_range).map(|x| (x, x + min_y_axis)), GREEN.filled())
                .point_size(2),
        )
        .unwrap();

    println!("\n\n========================\n\n");
    println!("slots in the current epoch {:?}", map.len());
    println!("slots assigned to jito: {:?}", entries.len());
    println!("current slot: {}", slot);
    println!("closet Jito slot: {}", future_leaders.iter().min().unwrap());
    println!(
        "closet Jito leader`s PubKey: {}",
        entries.get(future_leaders.iter().min().unwrap()).unwrap()
    );
    let time_left = (future_leaders.iter().min().unwrap() - slot) as f64 * 0.4;
    println!(
        "Approx. time to closet Jito leader: {:.2} sec or {:.2} min",
        time_left,
        time_left / 60.0
    );
    println!("\n\n========================\n\n");
}
