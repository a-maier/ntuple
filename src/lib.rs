mod bindings;
pub mod event;
pub mod ntuplewriter;

pub use ntuplewriter::NTupleWriter;
pub use event::Event;

include!(concat!(env!("OUT_DIR"), "/flags.rs"));

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
