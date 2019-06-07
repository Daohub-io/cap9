#[cfg(not(feature = "std"))]
use pwasm_std::Vec;
#[cfg(not(feature = "std"))]
use pwasm_std::String;
/// A listing is a category of import. There are 3 types of imports whitelisted,
/// greylisted, and blacklisted. There is no blacklist, everything that is not
/// whitlisted or greylisted is blacklisted, even if we don't recognise it.
///
///  * Whitelisted: Functions which can be run with no state effects and we
///      don't care about them. Examples include getting addresses, returning,
///      reverting etc.
///  * Greylisted: Functions that _do_ perform dangerous operations, but that we
///      need for the operation of syscalls etc. These calls need to be
///      surrounded by the correct protections. These are permitted to be
///      imported, but must be checked for safety.
///  * Blacklisted: Everything else. These cannot even be imported. If they are
///      imported the contract is not valid.
#[derive(Debug)]
pub enum Listing {
    White,
    Grey,
    Black,
}

pub trait Listed {
    fn listing(&self) -> Listing;
}

#[derive(Debug, Clone)]
pub struct ImportEntry {
    pub mod_name: String,
    pub field_name: String,
}

impl Listed for ImportEntry {
    fn listing(&self) -> Listing {
        // Nothing should need to be imported from outside "env", but let's
        // blacklist it just in case.
        if self.mod_name != "env" {
            Listing::Black
        } else {
            // Tehcnically we don't have to list blacklisted items here, but we
            // do just for clarity.
            match self.field_name.as_ref() {
                "memory" => Listing::White,
                "storage_read" => Listing::White,
                "storage_write" => Listing::Black,
                "ret" => Listing::White,
                "gas" => Listing::White,
                "input_length" => Listing::White,
                "fetch_input" => Listing::White,
                "panic" => Listing::White,
                "debug" => Listing::White,
                "ccall" => Listing::Black,
                "dcall" => Listing::Grey,
                "scall" => Listing::White,
                "value" => Listing::White,
                "create" => Listing::Black,
                "suicide" => Listing::White,
                "blockhash" => Listing::White,
                "blocknumber" => Listing::White,
                "coinbase" => Listing::White,
                "difficulty" => Listing::White,
                "gaslimit" => Listing::White,
                "timestamp" => Listing::White,
                "address" => Listing::White,
                "sender" => Listing::White,
                "origin" => Listing::White,
                "elog" => Listing::Black,
                "extcodesize" => Listing::White,
                "extcodecopy" => Listing::White,
                "create2" => Listing::Black,
                "gasleft" => Listing::White,
                _ => Listing::Black,
            }
        }
    }
}
