# Currency converter

Currency converter for CLI written in rust. The forex rates are obtained
from [ecb.europa.eu](https://www.ecb.europa.eu) and stored locally. Forex rates
are updated by ECB only once per day, therefore these are NOT LIVE
exchange rates.

## Installation

The recommended method to install it is download the binary from the 
[releases](https://github.com/sabarish-vm/currency_convert/releases/latest).
This also ensures that you download the completion files for bash and zsh.
If a release is not available for your system type, it can be cargo installed 
directly.

```shell
cargo install --git https://github.com/sabarish-vm/currency_convert.git
```

Alternative method is to git clone the repo, and cargo build it.
This also gives access to the completion files for bash and zsh.

```shell
git clone https://github.com/sabarish-vm/currency_convert.git
cd currency_convert
cargo build --release
```
