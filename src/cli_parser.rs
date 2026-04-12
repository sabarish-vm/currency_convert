use clap::{Arg, ArgAction, Command};

pub fn build_cli() -> Command {
    Command::new("curconv")
        .author("Sabarish github.com/sabarish-vm")
        .about(
            "\n\n
 ░███████  ░██    ░██ ░██░████  ░███████   ░███████  ░████████  ░██    ░██
░██    ░██ ░██    ░██ ░███     ░██    ░██ ░██    ░██ ░██    ░██ ░██    ░██
░██        ░██    ░██ ░██      ░██        ░██    ░██ ░██    ░██  ░██  ░██
░██    ░██ ░██   ░███ ░██      ░██    ░██ ░██    ░██ ░██    ░██   ░██░██
 ░███████   ░█████░██ ░██       ░███████   ░███████  ░██    ░██    ░███

 █▓▒▒░░░ 𝔸 𝕟𝕠-𝕟𝕠𝕟𝕤𝕖𝕟𝕤𝕖 𝕔𝕦𝕣𝕣𝕖𝕟𝕔𝕪 𝕔𝕠𝕟𝕧𝕖𝕣𝕥𝕖𝕣 𝕗𝕠𝕣 𝕥𝕙𝕖 𝕥𝕖𝕣𝕞𝕚𝕟𝕒𝕝 𝕝𝕠𝕧𝕖𝕣𝕤 ░░░▒▒▓█\n",
        )
        .override_usage(concat!(
            "There are three modes \n\n1) Update mode :",
            "To download the latest forex rates from www.ecb.europa.eu and store them locally\n\n",
            "\t\t curconv -u (or) curconv --update \n\n",
            "2) Conversion in CLI mode : To do forex conversions from CURRENCY1 to CURRENCY2,\n\n",
            "\t\t curconv AMOUNT CURRENCY1 CURRENCY2\n\n",
            "3) Terminal User Interface (TUI) mode : To do forex conversions using a TUI\n\n",
            "\t\t curconv -t (or) curconv --tui \n\n",
        ))
        .arg(
            Arg::new("tui")
                .short('t')
                .long("tui")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["AMOUNT", "CUR1", "CUR2"]),
        )
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
}
