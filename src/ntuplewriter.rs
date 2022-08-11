use std::{path::Path, ffi::CString, os::{unix::prelude::OsStrExt, raw::c_char}};

use crate::{bindings::{ntuple_create_writer, ntuple_write_event, ntuple_delete_writer, NTupleEvent, WriteResult}, Event};
use thiserror::Error;

#[derive(Debug)]
pub struct NTupleWriter (
    *mut crate::bindings::NTupleWriter
);

impl NTupleWriter {
    pub fn new<P: AsRef<Path>>(file: P, name: &str) -> Option<Self> {
        let file = CString::new(file.as_ref().as_os_str().as_bytes()).unwrap();
        let ptr = unsafe {
            ntuple_create_writer(
                file.as_ptr(),
                name.as_bytes().as_ptr() as *const c_char
            )
        };
        if ptr.is_null() {
            None
        } else {
            Some(Self(ptr))
        }
    }

    pub fn write(&mut self, event: &Event) -> Result<(), WriteError> {
        use WriteError::*;
        if event.nparticle < 0 {
            return Err(NegParticleNum(event.nparticle));
        }
        let npart = event.nparticle as usize;
        if event.px.len() != npart {
            return Err(LengthMismatch(event.px.len(), "px".to_string(), npart));
        }
        if event.py.len() != npart {
            return Err(LengthMismatch(event.py.len(), "py".to_string(), npart));
        }
        if event.pz.len() != npart {
            return Err(LengthMismatch(event.pz.len(), "pz".to_string(), npart));
        }
        if event.energy.len() != npart {
            return Err(LengthMismatch(event.energy.len(), "energy".to_string(), npart));
        }
        if event.pdg_code.len() != npart {
            return Err(LengthMismatch(event.pdg_code.len(), "pdg_code".to_string(), npart));
        }
        if event.user_weights.len() > i32::MAX as usize {
            return Err(TooManyWeights);
        }

        let event = NTupleEvent {
            id: event.id,
            nparticle: event.nparticle,
            px: event.px.as_ptr(),
            py: event.py.as_ptr(),
            pz: event.pz.as_ptr(),
            energy: event.energy.as_ptr(),
            alphas: event.alphas,
            kf: event.pdg_code.as_ptr(),
            weight: event.weight,
            weight2: event.weight2,
            me_wgt: event.me_weight,
            me_wgt2: event.me_weight2,
            x1: event.x1,
            x2: event.x2,
            x1p: event.x1p,
            x2p: event.x2p,
            id1: event.id1,
            id2:  event.id2,
            fac_scale: event.fac_scale,
            ren_scale: event.ren_scale,
            nuwgt: event.user_weights.len() as i32,
            usr_wgts: event.user_weights.as_ptr(),
            part: event.part.into(),
            alphas_power: event.alphas_power,
        };
        let res = unsafe {
            ntuple_write_event(self.0, &event)
        };
        match res {
            WriteResult::OK => Ok(()),
            err => Err(WriteError::from(err))
        }
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum WriteError {
    #[error("Too many particles in event")]
    TooManyParticles,
    #[error("Too many weights in event")]
    TooManyWeights,
    #[error("Length `{1}` of `{0}` does not match number of particles `{2}`")]
    LengthMismatch(usize, String, usize),
    #[error("Number of particles is negative: `{0}`")]
    NegParticleNum(i32),
    #[error("Error filling event into TTree")]
    FillError,

    #[error("Unknown error")]
    UnknownError,
}

impl From<WriteResult> for WriteError {
    fn from(r: WriteResult) -> Self {
        match r {
            WriteResult::TOO_MANY_PARTICLES => Self::TooManyParticles,
            WriteResult::TOO_MANY_WEIGHTS => Self::TooManyWeights,
            WriteResult::FILL_ERROR => Self::FillError,
            _ => Self::UnknownError
        }
    }
}

impl Drop for NTupleWriter {
    fn drop(&mut self) {
        unsafe { ntuple_delete_writer(self.0) }.into()
    }
}
