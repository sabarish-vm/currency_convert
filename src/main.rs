use clap::{Arg, ArgAction, Command};
use core::panic;
use std::path::{Path, PathBuf};

mod functions;
mod structs;
use functions::{converter, csv_parser, downloader, read_file_contents, unzipper};
use structs::DataPath;

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
        zip: &zip_path,
        csv: &csv_path,
        dir: &output_dir,
    };
    if !paths.csv.exists() {
        downloader(&paths);
        unzipper(&paths);
    }
    let contents = read_file_contents(&paths).unwrap();
    let (currencies, map) = csv_parser(&contents, &paths);

    let date = match map.get("Date") {
        Some(x) => x.get_string().unwrap(),
        None => panic!("Date key not found in the forex rate data file"),
    };

    let matches = Command::new("curconv")
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
        let (_, map2) = csv_parser(&contents, &paths);
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
