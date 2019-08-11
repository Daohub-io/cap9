/// Module for handling the connection to a local Ethereum node.
use web3::Transport;
use web3::types::{Address};
use web3::futures::Future;
use pwasm_abi::types::*;


pub struct EthConn<T: Transport> {
    pub web3: web3::api::Web3<T>,
    pub sender: Address,
    pub eloop: web3::transports::EventLoopHandle,
}

impl<T: Transport> EthConn<T> {

}

impl EthConn<web3::transports::Http> {
    pub fn new_http() -> Self {
        let (eloop, transport) = web3::transports::Http::new("http://localhost:8545").expect("No network");
        // eloop.intos_remote();
        let web3 = web3::Web3::new(transport);
        let sender = create_account(String::from("user"), String::from("user"));
        EthConn {
            web3,
            sender,
            eloop,
        }
    }
}

    // The two Nones are for user/pass for authentication

fn create_account(name: String, password: String) -> Address {
    let name_json = serde_json::to_value(name).unwrap();
    let password_json = serde_json::to_value(password).unwrap();
    let client = jsonrpc::client::Client::new(String::from("http://localhost:8545"), None, None);
    let params = &[name_json, password_json];
    let request = client.build_request("parity_newAccountFromPhrase", params);
    match client.send_request(&request).and_then(|res| res.into_result::<Address>()) {
        Ok(x) => x,
        Err(e) => panic!("{:?}", e),
    }
}


pub fn list_storage_keys(address: Address) -> jsonrpc::Response {
    let address_json = serde_json::to_value(address).unwrap();
    println!("address_json: {:?}", address_json);
    let client = jsonrpc::client::Client::new(String::from("http://localhost:8545"), None, None);
    let params = &[address_json, serde_json::to_value(100).unwrap(), serde_json::Value::Null];
    let request = client.build_request("parity_listStorageKeys", params);
    println!("request: {:?}", request);
    match client.send_request(&request) {
        Ok(x) => x,
        Err(e) => panic!("{:?}", e),
    }
}
