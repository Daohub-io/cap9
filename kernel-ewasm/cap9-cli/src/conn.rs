use web3::Transport;
use web3::types::{Address};
use web3::futures::Future;

pub struct EthConn<T: Transport> {
    pub web3: web3::api::Web3<T>,
    pub sender: Address,
}

impl<T: Transport> EthConn<T> {
    pub fn new_http() -> EthConn<web3::transports::Http> {
        let (_eloop, transport) = web3::transports::Http::new("http://localhost:8545").expect("No network");
        let web3 = web3::Web3::new(transport);
        let sender = match web3.eth().accounts().wait() {
            Err(_r) => {
                println!("No Ethereum network available");
                std::process::exit(1);
                },
            Ok(x) => x[0],
        };
        EthConn {
            web3,
            sender
        }
    }
}
