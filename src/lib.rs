mod bindings;
pub mod event;
pub mod ntuplereader;
pub mod ntuplewriter;
#[cfg(feature = "hepmc2")]
pub mod conv;

pub use crate::ntuplereader::NTupleReader;
pub use crate::ntuplewriter::NTupleWriter;
pub use crate::event::Event;

include!(concat!(env!("OUT_DIR"), "/flags.rs"));

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, fs::read_dir};
    use ntuplereader::NTupleReader;
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test() {
        let mut data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        data_path.push("test_data");

        for root_file in read_dir(data_path).unwrap() {
            let root_file = root_file.unwrap();

            let tmp1 = NamedTempFile::new().unwrap();
            let tmp2 = NamedTempFile::new().unwrap();

            let reader = NTupleReader::new(root_file.path()).unwrap();
            {
                let mut writer = NTupleWriter::new(tmp1.path(), "").unwrap();
                for event in reader {
                    writer.write(&event.unwrap()).unwrap();
                }
            }

            let reader = NTupleReader::new(tmp1.path()).unwrap();
            {
                let mut writer = NTupleWriter::new(tmp2.path(), "").unwrap();
                for event in reader {
                    writer.write(&event.unwrap()).unwrap();
                }
            }

            let reader1 = NTupleReader::new(tmp1.path()).unwrap();
            let mut reader2 = NTupleReader::new(tmp2.path()).unwrap();

            for event1 in reader1 {
                let event2 = reader2.next().unwrap();
                assert_eq!(event1.unwrap(), event2.unwrap())

            }
        }
    }
}
