#![feature(async_closure)]
#![feature(array_methods)]

pub mod dapp;
pub mod header_relay;
pub mod transfer;
pub mod util;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
