# Currency converter

Currency converter for CLI written in rust. The forex rates are obtained from <www.ecb.europa.eu> and stored locally. Forex rates are updated by ECB only once per day, therefore these are NOT LIVE exchange rates.

## Installation

Easy method is to cargo install the repo directly.

```shell
cargo install --git https://github.com/sabarish-vm/currency_convert.git
```

Recommended method is to git clone the repo, and cargo build it.
This also gives access to the completion files for bash and zsh.

```shell
git clone https://github.com/sabarish-vm/currency_convert.git
cd currency_convert
cargo build --release
```
