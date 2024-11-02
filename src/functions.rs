use core::panic;
use curl::easy::Easy;
use std::collections::HashMap;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::num::ParseFloatError;
use zip::ZipArchive;

use crate::structs::{DataPath, Value};

pub fn read_file_contents(paths: &DataPath) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let string_content = match read_to_string(paths.csv) {
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

pub fn csv_parser<'a>(
    contents: &'a [String],
    paths: &DataPath,
) -> (String, HashMap<&'a str, Value<'a>>) {
    let headers: Vec<&str> = match contents.first() {
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
        .first()
        .unwrap()
        .clone()
        .replace("Date,", "")
        .replace(',', "");
    let size = headers.len();
    let mut fxrates_hash: HashMap<&str, Value> = HashMap::new();
    for index in 0..size {
        let key = headers.get(index).unwrap().trim();
        let value = values.get(index).unwrap().trim();
        let fvalue: Result<f64, ParseFloatError> = value.parse();
        let final_value = match fvalue {
            Ok(val) => Value::Float(val),
            Err(_) => Value::Str(value),
        };
        fxrates_hash.insert(key, final_value);
        fxrates_hash.insert("EUR", Value::Float(1.0));
    }
    (currencies, fxrates_hash)
}

pub fn downloader(paths: &DataPath) {
    let mut file = File::create(paths.zip).unwrap();

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

pub fn unzipper(paths: &DataPath) {
    let file = File::open(paths.zip).unwrap();
    let mut archive = ZipArchive::new(file)
        .unwrap_or_else(|x| panic!("\nProblems unzipping the downloaded file. {}\n", x));
    archive.extract(paths.dir).unwrap();
}

pub fn converter(amt: &f64, from: &str, to: &str, map: &HashMap<&str, Value>, currencies: &str) {
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
