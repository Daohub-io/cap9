# If the parity-ethereum directory does not exist, clone it
# pwd
# ls
# if [ ! -d parity-ethereum ]
# then
#     echo "Parity not installed, cloning..."
#     git clone https://github.com/Daohub-io/parity-ethereum.git
# fi
# cd parity-ethereum
# git fetch --all
# git checkout master
# cargo build -j 1
# cargo build --verbose --release --features final
# strip target/debug/parity
# file target/debug/parity
if parity --version; then
    echo "Parity node installed"
else
    cargo install --git https://github.com/Daohub-io/parity-ethereum.git --bin parity -j 1 --bin parity parity-ethereum
fi