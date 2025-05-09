use std::{
    ffi::CString,
    os::unix::prelude::OsStrExt,
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::{
    bindings::{
        ntuple_create_reader, ntuple_delete_reader, ntuple_num_events,
        ntuple_read_event, NTupleCreateError, NTupleReadStatus,
        NTupleReaderCreateResult,
    },
    Event,
};

#[derive(Debug)]
pub struct Reader {
    reader: *mut crate::bindings::NTupleReader,
    idx: i64,
}

impl Reader {
    pub fn new<P: AsRef<Path>>(file: P) -> Result<Self, CreateError> {
        let filename = file.as_ref();
        let file = match CString::new(filename.as_os_str().as_bytes()) {
            Ok(f) => f,
            Err(err) => panic!(
                "Failed to create nTuple Reader from {filename:?}: Found nul byte at position {} in filename",
                err.nul_position()
            )
        };
        let NTupleReaderCreateResult { reader, error } =
            unsafe { ntuple_create_reader(file.as_ptr()) };
        if reader.is_null() {
            let err = match error {
                NTupleCreateError::OPEN_FAILED => {
                    CreateError::Open(filename.to_path_buf())
                }
                NTupleCreateError::NO_TTREE => CreateError::NoTTree,
                NTupleCreateError::EXCEPTION => CreateError::Exception,
                _ => CreateError::Unknown,
            };
            Err(err)
        } else {
            debug_assert_eq!(error, NTupleCreateError::NONE);
            Ok(Self { reader, idx: 0 })
        }
    }

    pub fn nevent(&self) -> &i64 {
        &self.idx
    }

    pub fn nevent_mut(&mut self) -> &mut i64 {
        &mut self.idx
    }
}

impl Iterator for Reader {
    type Item = Result<Event, ReadError>;

    fn next(&mut self) -> Option<Self::Item> {
        use self::ReadError::*;
        let res = unsafe { ntuple_read_event(self.reader, self.idx) };
        if res.status != NTupleReadStatus::READ_NO_ENTRY {
            self.idx += 1;
        }
        match res.status {
            NTupleReadStatus::READ_OK => Some(Ok(res.event.into())),
            NTupleReadStatus::READ_NO_ENTRY => None,
            NTupleReadStatus::READ_ERROR => Some(Err(ReadError)),
            NTupleReadStatus::READ_EXCEPTION => Some(Err(Exception)),
            NTupleReadStatus::READ_TOO_MANY_PARTICLES => {
                Some(Err(TooManyParticles(res.event.nparticle)))
            }
            NTupleReadStatus::READ_NEGATIVE_NUMBER_OF_PARTICLES => {
                Some(Err(NegParticleNum(res.event.nparticle)))
            }
            NTupleReadStatus::READ_TOO_MANY_WEIGHTS => {
                Some(Err(TooManyWeights(res.event.nuwgt)))
            }
            NTupleReadStatus::READ_NEGATIVE_NUMBER_OF_WEIGHTS => {
                Some(Err(NegWeightNum(res.event.nuwgt)))
            }
            _ => Some(Err(Unknown)),
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.idx += n as i64;
        self.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let evs = unsafe { ntuple_num_events(self.reader) };
        let remaining = (evs - self.idx) as usize;
        (remaining, Some(remaining))
    }

    fn last(mut self) -> Option<Self::Item> {
        let evs = unsafe { ntuple_num_events(self.reader) };
        if evs > 0 {
            self.idx = evs - 1;
            self.next()
        } else {
            None
        }
    }
}

impl ExactSizeIterator for Reader {}

impl Drop for Reader {
    fn drop(&mut self) {
        unsafe { ntuple_delete_reader(self.reader) }
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ReadError {
    #[error("Too many particles in event")]
    TooManyParticles(i32),
    #[error("Too many weights in event")]
    TooManyWeights(i32),
    #[error("Number of particles is negative: `{0}`")]
    NegParticleNum(i32),
    #[error("Number of user weights is negative: `{0}`")]
    NegWeightNum(i32),
    #[error("Read error")]
    ReadError,
    #[error("Encountered an exception during reading")]
    Exception,

    #[error("Unknown error")]
    Unknown,
}

#[derive(Clone, Debug, Error, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum CreateError {
    #[error("Failed to open {0:?}")]
    Open(PathBuf),
    #[error("Failed to find a `TTree` named \"BHSntuples\"")]
    NoTTree,
    #[error("Encountered an exception during creation")]
    Exception,

    #[error("Unknown error")]
    Unknown,
}
