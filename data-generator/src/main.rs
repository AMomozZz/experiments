use std::collections::VecDeque;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use clap::Parser;
use csv::WriterBuilder;
use nexmark::config::NexmarkConfig;
use nexmark::event::Event;
use nexmark::event::EventType;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use base64::{engine::general_purpose::STANDARD, Engine};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct WasmComponent {
    #[serde(serialize_with = "serialize_vec_u8", deserialize_with = "deserialize_vec_u8")]
    pub file: Vec<u8>,
    pub pkg_name: String,
    pub name: String,
    pub date_time: u64,
    pub extra: String,
}

fn serialize_vec_u8<S>(vec: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let encoded = STANDARD.encode(vec);
    serializer.serialize_str(&encoded)
}

fn deserialize_vec_u8<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let encoded: String = Deserialize::deserialize(deserializer)?;
    STANDARD.decode(&encoded).map_err(serde::de::Error::custom)
}

impl WasmComponent {
    fn new_empty(e: Event) -> Self {
        Self {
            file: vec![],
            pkg_name: String::new(),
            name: String::new(),
            date_time: e.timestamp(),
            extra: String::new(),
        }
    }

    fn empty(&self) -> bool {
        self.pkg_name.is_empty() && self.name.is_empty() && self.file.is_empty()
    }

    fn new(e: Event, file: &PathBuf, pkg_name: &str, name: &str) -> Self {
        let mut wc = WasmComponent::new_empty(e);
        // eprintln!("{:?}", file);
        wc.file = fs::read(file).unwrap();
        wc.pkg_name = pkg_name.to_string();
        wc.name = name.to_string();
        wc
    }
}

#[derive(Parser, Clone, Debug)]
struct Args {
    /// Number of events to generate.
    #[clap(long, default_value = "2000000")]
    num_events: usize,
    #[clap(long, default_value_t = false)]
    persons: bool,
    #[clap(long, default_value_t = false)]
    auctions: bool,
    #[clap(long, default_value_t = false)]
    bids: bool,
    #[clap(long, default_value_t = false)]
    components: bool,
    #[clap(long, default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/components/"))]
    wasm_dir: PathBuf,
    #[clap(long, default_value = "pkg:component/nexmark")]
    pkg_name: String,
    #[clap(long, default_value = "e2")]
    name: String,
    #[clap(long, default_value = "100")]
    each: usize,
    #[clap(long, default_value = ".")]
    dir: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let conf = NexmarkConfig {
        // Set to 1700000000 (Tue Nov 14 2023 22:13:20 GMT+0000) to make events reproducible.
        base_time: 1700000000,
        ..Default::default()
    };
    let mut total = 0;
    if args.persons {
        total += conf.person_proportion;
    }
    if args.auctions {
        total += conf.auction_proportion;
    }
    if args.bids {
        total += conf.bid_proportion;
    }
    if total == 0 {
        return Err("At least one of --bids, --auctions, --persons must be set".into());
    }
    std::fs::create_dir_all(&args.dir)?;
    std::fs::create_dir_all(&args.wasm_dir)?;

    let mut wasm_files = Vec::new();
    if args.components {
        for entry in fs::read_dir(args.wasm_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "wasm" {
                        wasm_files.push(path);
                    }
                }
            }
        }
    }

    for (name, ty, flag, proportion) in [
        ("persons", EventType::Person, args.persons, conf.person_proportion),
        ("auctions", EventType::Auction, args.auctions, conf.auction_proportion),
        ("bids", EventType::Bid, args.bids, conf.bid_proportion),
    ] {
        if flag == false {
            continue;
        }
        let n = args.num_events * proportion / total;
        println!("Generating {}*{}/{} = {} events", args.num_events, proportion, total, n);

        let mut components_event_queue: VecDeque<Event> = VecDeque::new();

        let file = File::create(args.dir.join(name).with_extension("csv"))?;
        let mut writer = WriterBuilder::new()
            .has_headers(false)
            .from_writer(BufWriter::new(file));
        nexmark::EventGenerator::new(conf.clone())
            .with_type_filter(ty)
            .take(n)
            .enumerate()
            .inspect(|(i, e)| {
                let m = i + 1;
                let p = n / 100;
                if m % p == 10 {
                    let progress = m / p;
                    println!("{name}: {progress}%");
                }
                if m % args.each == 0 && args.components == true {
                    components_event_queue.push_back(e.clone());
                }
            })
            .try_for_each(|(_, event)| match event {
                Event::Person(row) if ty == EventType::Person => writer.serialize(&row),
                Event::Auction(row) if ty == EventType::Auction => writer.serialize(&row),
                Event::Bid(row) if ty == EventType::Bid => writer.serialize(&row),
                _ => unreachable!(),
            })?;

        let component_file = File::create(args.dir.join(format!("component_{}", name)).with_extension("csv")).unwrap();
        let mut component_writer = WriterBuilder::new()
            .has_headers(false)
            .flexible(true)
            .from_writer(BufWriter::new(component_file));
        // Generate and write component events
        let n = components_event_queue.len();
        eprintln!("{:?}", wasm_files);
        components_event_queue
        .into_iter()
        .enumerate()
        .inspect(|(i, _)| {
            let p = n / 100;
            if i % p == 0 {
                let progress = i / p;
                println!("{}: {}%", format!("component_{}", name), progress);
            }
        })
        .map(|(_, e)| {
            if wasm_files.is_empty() {
                unreachable!("no wasm files");
            } else {
                let mut rng = rand::rng();
                match wasm_files.choose(&mut rng) {
                    Some(random_file) => WasmComponent::new(e, &random_file, &args.pkg_name, &args.name),
                    None => {
                        unreachable!("no random .wasm");
                    },
                }
            }
        })
        .try_for_each(|component| match component.empty() {
            false => component_writer.serialize(&component),
            _ => unreachable!(),
        })?;
    }

    Ok(())
}
