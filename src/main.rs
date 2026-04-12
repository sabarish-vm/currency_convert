use core::panic;
use std::io;
use std::path::{Path, PathBuf};

mod cli_parser;
mod data_strcuts;
mod functions;
mod tui_app;
mod tui_fuzzy_widget;

use data_strcuts::DataPath;
use functions::{converter, csv_parser, downloader, read_file_contents, unzipper};

use crate::cli_parser::build_cli;

fn main() -> io::Result<()> {
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

    let matches = build_cli().get_matches();
    let tui_flag = matches.get_flag("tui");
    if tui_flag {
        let cur: Vec<&str> = currencies.split(' ').filter(|s| s.len() > 1).collect();
        let mut app: tui_app::App = tui_app::App::new(cur, map);
        ratatui::run(|terminal| app.run(terminal))
    } else {
        let upd = matches.get_flag("update");
        if upd {
            downloader(&paths);
            unzipper(&paths);
            let (_, map2) = csv_parser(&contents, &paths);
            let date2 = map2.get("Date").unwrap().get_string().unwrap();
            println!("Updated to the forex rates as on {}", date2);
            println!("The rates are updated by ECB once per day at 16:00 CET(CEST)");
        }
        if let (Some(amount), Some(inpcur), Some(tocur)) = (
            matches.get_one::<f64>("AMOUNT"),
            matches.get_one::<String>("CUR1"),
            matches.get_one::<String>("CUR2"),
        ) {
            println!("Using forex rates as on {}", date);
            converter(amount, inpcur, tocur, &map, currencies);
            Ok(())
        } else {
            Ok(())
        }
    }
}
