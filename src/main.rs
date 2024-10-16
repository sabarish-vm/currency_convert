use clap::{Arg, ArgAction, Command};
use core::panic;
use curl::easy::Easy;
use std::collections::HashMap;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::num::ParseFloatError;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

struct DataPath {
    zip: PathBuf,
    csv: PathBuf,
    dir: PathBuf,
}

#[derive(Debug)]
enum Value {
    Float(f64),
    Str(String),
}
impl Value {
    fn get_string(&self) -> Option<&String> {
        if let Value::Str(s) = self {
            Some(s)
        } else {
            None
        }
    }
}

fn read_file_contents(paths: &DataPath) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let string_content = match read_to_string(&paths.csv) {
        Ok(x) => x,
        Err(e) => {
            downloader(paths);
            unzipper(paths);
            panic!("Error occured in reading the data file : {}", e)
        }
    };
    let lines: Vec<String> = string_content.lines().map(|s| s.to_string()).collect();
    Ok(lines)
}

fn csv_parser(paths: &DataPath) -> (String, HashMap<String, Value>) {
    let contents = read_file_contents(paths).unwrap();
    let headers: Vec<&str> = match contents.get(0) {
        Some(x) => x.split(',').collect(),
        None => {
            downloader(paths);
            unzipper(paths);
            panic!("\nRestoring data file... Please re-run the command\n");
        }
    };
    let values: Vec<&str> = match contents.get(1) {
        Some(x) => x.split(',').collect(),
        None => {
            downloader(paths);
            unzipper(paths);
            panic!("\nRestoring data file... Please re-run the command\n");
        }
    };
    if headers.len() != values.len() {
        downloader(paths);
        unzipper(paths);
        panic!("\nSomething is wrong with the data file. Restoring data file... Please re-run the command\n");
    }
    let currencies = contents
        .get(0)
        .unwrap()
        .clone()
        .replace("Date,", "")
        .replace(',', "");
    let size = headers.len();
    let mut fxrates_hash: HashMap<String, Value> = HashMap::new();
    for index in 0..size {
        let key = headers.get(index).unwrap().trim().to_string();
        let value = values.get(index).unwrap().trim().to_string();
        let fvalue: Result<f64, ParseFloatError> = value.parse();
        let final_value = match fvalue {
            Ok(val) => Value::Float(val),
            Err(_) => Value::Str(value),
        };
        fxrates_hash.insert(key, final_value);
        fxrates_hash.insert("EUR".to_string(), Value::Float(1.0));
    }
    (currencies, fxrates_hash)
}

fn downloader(paths: &DataPath) {
    let mut file = File::create(&paths.zip).unwrap();

    let url = "https://www.ecb.europa.eu/stats/eurofxref/eurofxref.zip";
    let mut easy = Easy::new();
    easy.url(url).unwrap();
    let mut transfer = easy.transfer();
    transfer
        .write_function(|data| {
            file.write_all(data).unwrap();
            Ok(data.len())
        })
        .unwrap();
    transfer.perform().unwrap();
}

fn unzipper(paths: &DataPath) {
    let file = File::open(&paths.zip).unwrap();
    let mut archive = ZipArchive::new(file)
        .unwrap_or_else(|x| panic!("\nProblems unzipping the downloaded file. {}\n", x));
    archive.extract(&paths.dir).unwrap();
}

fn converter(amt: &f64, from: &str, to: &str, map: &HashMap<String, Value>, currencies: &String) {
    if let (Some(Value::Float(fac_from)), Some(Value::Float(fac_to))) = (map.get(from), map.get(to))
    {
        let res = amt * fac_to / fac_from;
        println!("{} {} = {} {}", amt, from, res, to);
    } else {
        println!(
            concat!(
                "One or both currencies passed is not understood. ",
                "Available currencies are :\n",
                "{}"
            ),
            currencies.trim()
        );
    }
}

fn main() {
    let home_dir_opt = dirs::home_dir();
    let output_dir: PathBuf = home_dir_opt
        .unwrap()
        .join(Path::new(".config"))
        .join(Path::new(".curconv"));
    if !output_dir.exists() {
        let _ = std::fs::create_dir(&output_dir);
    }
    let zip_path = output_dir.join(Path::new("eurofxref.zip"));
    let csv_path = output_dir.join(Path::new("eurofxref.csv"));
    let paths = DataPath {
        zip: zip_path,
        csv: csv_path,
        dir: output_dir,
    };
    if !paths.csv.exists() {
        downloader(&paths);
        unzipper(&paths);
    }
    let (currencies, map) = csv_parser(&paths);

    let date = match map.get("Date") {
        Some(x) => x.get_string().unwrap(),
        None => panic!("Date key not found in the forex rate data file"),
    };

    let matches = Command::new("curconv")
        .version("1.0.0")
        .author("Sabarish github.com/sabarish-vm")
        .about("A simple currency converter to use from the terminal")
        .override_usage(concat!(
            "There are two modes \n\n1) Update mode :",
            "To download the latest forex rates from www.ecb.europa.eu and store them locally\n\n",
            "\t\t curconv -u (or) curconv --update \n\n",
            "2) Conversion mode : To do forex conversions from CURRENCY1 to CURRENCY2,\n\n",
            "\t\t curconv AMOUNT CURRENCY1 CURRENCY2",
        ))
        .arg(
            Arg::new("update")
                .short('u')
                .long("update")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["AMOUNT", "CUR1", "CUR2"]),
        )
        .arg(
            Arg::new("AMOUNT")
                .value_parser(clap::value_parser!(f64))
                .required(true)
                .index(1),
        )
        .arg(Arg::new("CUR1").required(true).index(2))
        .arg(Arg::new("CUR2").required(true).index(3))
        .get_matches();
    let upd = matches.get_flag("update");
    if upd {
        downloader(&paths);
        unzipper(&paths);
        let (_, map2) = csv_parser(&paths);
        let date2 = map2.get("Date").unwrap().get_string().unwrap();
        println!("Updated to the forex rates as on {}", date2);
        println!("The rates are updated by ECB once per day at 16:00 CET(CEST)");
        std::process::exit(0)
    }
    if let (Some(amount), Some(inpcur), Some(tocur)) = (
        matches.get_one::<f64>("AMOUNT"),
        matches.get_one::<String>("CUR1"),
        matches.get_one::<String>("CUR2"),
    ) {
        println!("Using forex rates as on {}", date);
        converter(amount, inpcur, tocur, &map, &currencies)
    } else {
        println!("Missing args, please provide args as in the format\n \tcurconv AMOUNT CURRENCY1 --to CURRENCY2\n Use --help for more information");
        std::process::exit(1)
    }
}
