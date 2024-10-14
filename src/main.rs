use clap::{Arg, ArgAction, Command, Parser};
use curl::easy::Easy;
use std::collections::HashMap;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::num::ParseFloatError;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

#[derive(Debug)]
enum Value {
    Float(f64),
    Str(String),
}
impl Value {
    fn get_string(&self) -> Option<&String> {
        if let Value::Str(s) = self {
            Some(s) // Returning a reference to the integer
        } else {
            None
        }
    }
    // fn get_float(&self) -> Option<&f64> {
    //     if let Value::Float(f) = self {
    //         Some(f) // Returning a reference to the integer
    //     } else {
    //         None
    //     }
    // }
}

fn read_file_contents(path: &PathBuf) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let string_content = read_to_string(path).unwrap();
    let lines: Vec<String> = string_content.lines().map(|s| s.to_string()).collect();
    Ok(lines)
}

fn csv_parser(path: &PathBuf) -> HashMap<String, Value> {
    let contents = read_file_contents(path).unwrap();
    let headers: Vec<&str> = contents.get(0).unwrap().split(',').collect();
    let values: Vec<&str> = contents.get(1).unwrap().split(',').collect();
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
    fxrates_hash
}

fn downloader(path: &PathBuf) {
    let mut file = File::create(path).unwrap();

    let url = "https://www.ecb.europa.eu/stats/eurofxref/eurofxref.zip";
    let mut easy = Easy::new();
    easy.url(url).unwrap();
    let mut transfer = easy.transfer();
    transfer
        .write_function(|data| {
            // Method 1 : More clean to read, look up curl-4.47 docs
            // match file.write_all(data) {
            //     Ok(_) => Ok(data.len()),
            //     Err(_) => Err(curl::easy::WriteError::Pause),
            // }
            // Method 2 : Lesser number of lines and a bit tricky to understand
            // First line below either returns an erro through unwrap()
            // or Second line returns usize if no errors occur
            // Thus both methods are equivalent
            file.write_all(data).unwrap();
            Ok(data.len())
        })
        .unwrap();
    transfer.perform().unwrap();
}

fn unzipper(path: &PathBuf, output_dir: &PathBuf) {
    let file = File::open(path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    archive.extract(output_dir).unwrap();
}

fn converter(amt: &f64, from: &str, to: &str, map: &HashMap<String, Value>) {
    if let (Some(Value::Float(fac_from)), Some(Value::Float(fac_to))) = (map.get(from), map.get(to))
    {
        let res = amt * fac_to / fac_from;
        println!("{} {} = {} {}", amt, from, res, to);
    } else {
        println!("One or both keys do not have a Float value.");
    }
    //    //    let _mto = map.get(from) also works for some reason
    //    let _mfrom = &map[from];
    //    let _mto = &map[to];
    //    let res: f64;
    //    let _fac_to = match _mto {
    //        Value::Float(f) => Some(f),
    //        Value::Str(s) => None,
    //    };
    //    let _fac_from = match _mfrom {
    //        Value::Float(f) => Some(f),
    //        Value::Str(s) => None,
    //    };
    //    if let Some(fac_to) = _fac_to {
    //        if let Some(fac_from) = _fac_from {
    //            res = fac_from + fac_to
    //        }
    //    };
}

#[derive(Parser)]
#[command(name = "Curconv")]
#[command(about = "Currency converter application", long_about = None)]
struct Cli {
    #[arg(short, long,action = clap::ArgAction::SetTrue,group="update")]
    update: Option<bool>,

    #[arg(group = "conv")]
    amount: f64,

    #[arg(group = "conv")]
    inpcur: String,

    #[arg(group = "conv")]
    #[arg(short, long)]
    to: String,
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
    let map = csv_parser(&csv_path);

    let date = match map.get("Date") {
        Some(x) => x.get_string().unwrap(),
        None => panic!("Date key in the forex rate CSV file"),
    };

    let matches = Command::new("myprog")
        .arg(
            Arg::new("update")
                .short('u')
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["amount", "CUR1", "CUR2"]),
        )
        .arg(Arg::new("amount").value_parser(clap::value_parser!(f64)))
        .arg(Arg::new("CUR1"))
        .arg(Arg::new("CUR2").short('t').long("to"))
        .get_matches();
    // println!("A");
    let upd = matches.get_flag("update");
    if upd {
        downloader(&zip_path);
        unzipper(&zip_path, &output_dir);
        println!("Update successful");
        std::process::exit(0)
    }
    // print!("B");
    if let (Some(amount), Some(inpcur), Some(tocur)) = (
        matches.get_one::<f64>("amount"),
        matches.get_one::<String>("CUR1"),
        matches.get_one::<String>("CUR2"),
    ) {
        println!("Using forex rates as on {}", date);
        converter(amount, inpcur, tocur, &map)
    }
    // let args = Cli::parse();
    // if let Some(upd) = args.update {
    //     println!("{}", upd);
    //     downloader(&zip_path);
    //     unzipper(&zip_path, &output_dir);
    //     std::process::exit(0)
    // }
    // if let (amount, inpcur, tocur) = (args.amount, args.inpcur, args.to) {
    //     converter(&(amount), &(inpcur), &(tocur), &map)
    // } else {
    //     println!("Either run it in update mode by using --update or -u. Or run it in conversion mode by passing args like \n\tAMT CUR1 --to CUR2")
    // }
}
