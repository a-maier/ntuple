use core::slice;

use thiserror::Error;

use crate::bindings::NTupleEvent;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Event {
    pub id: i32,
    pub nparticle: i32,
    pub px: Vec<f32>,
    pub py: Vec<f32>,
    pub pz: Vec<f32>,
    pub energy: Vec<f32>,
    pub alphas: f64,
    pub pdg_code: Vec<i32>,
    pub weight: f64,
    pub weight2: f64,
    pub me_weight: f64,
    pub me_weight2: f64,
    pub x1: f64,
    pub x2: f64,
    pub x1p: f64,
    pub x2p: f64,
    pub id1: i32,
    pub id2: i32,
    pub fac_scale: f64,
    pub ren_scale: f64,
    pub user_weights: Vec<f64>,
    pub part: Part,
    pub alphas_power: i16,
}

impl From<NTupleEvent> for Event {
    fn from(ev: NTupleEvent) -> Self {
        assert!(ev.nparticle >= 0);
        assert!(ev.nuwgt >= 0);
        let npart = ev.nparticle as usize;
        let nwgt = ev.nuwgt as usize;
        let part = match ev.part.try_into() {
            Ok(part) => part,
            Err(err) => panic!("Unrecognised event type: {err}"),
        };
        Self {
            id: ev.id,
            nparticle: ev.nparticle,
            px: unsafe { slice::from_raw_parts(ev.px, npart) }.to_owned(),
            py: unsafe { slice::from_raw_parts(ev.py, npart) }.to_owned(),
            pz: unsafe { slice::from_raw_parts(ev.pz, npart) }.to_owned(),
            energy: unsafe { slice::from_raw_parts(ev.energy, npart) }
                .to_owned(),
            alphas: ev.alphas,
            pdg_code: unsafe { slice::from_raw_parts(ev.kf, npart) }.to_owned(),
            weight: ev.weight,
            weight2: ev.weight2,
            me_weight: ev.me_wgt,
            me_weight2: ev.me_wgt2,
            x1: ev.x1,
            x2: ev.x2,
            x1p: ev.x1p,
            x2p: ev.x2p,
            id1: ev.id1,
            id2: ev.id2,
            fac_scale: ev.fac_scale,
            ren_scale: ev.ren_scale,
            user_weights: unsafe { slice::from_raw_parts(ev.usr_wgts, nwgt) }
                .to_owned(),
            part,
            alphas_power: ev.alphas_power,
        }
    }
}

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Part {
    #[default]
    B,
    I,
    R,
    V,
    S,
}

impl From<Part> for u8 {
    fn from(p: Part) -> Self {
        use Part::*;
        match p {
            B => b'B',
            I => b'I',
            R => b'R',
            V => b'V',
            S => b'S',
        }
    }
}

impl From<Part> for i8 {
    fn from(p: Part) -> Self {
        u8::from(p) as i8
    }
}

impl From<Part> for char {
    fn from(p: Part) -> Self {
        u8::from(p) as char
    }
}

#[derive(Copy, Clone, Debug, Error, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ConversionError {
    // We also support 'S', but don't want to encourage users to use
    // it, as it is not an official event type according to
    // arXiv:1310.7439
    #[error("'{0}' is not one of 'B', 'I', 'R', 'V'")]
    BadChar(char),
}

impl TryFrom<char> for Part {
    type Error = ConversionError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        use Part::*;
        match c {
            'B' => Ok(B),
            'I' => Ok(I),
            'R' => Ok(R),
            'V' => Ok(V),
            'S' => Ok(S),
            c => Err(ConversionError::BadChar(c)),
        }
    }
}

impl TryFrom<u8> for Part {
    type Error = ConversionError;

    fn try_from(c: u8) -> Result<Self, Self::Error> {
        use Part::*;
        match c {
            b'B' => Ok(B),
            b'I' => Ok(I),
            b'R' => Ok(R),
            b'V' => Ok(V),
            b'S' => Ok(S),
            c => Err(ConversionError::BadChar(c.into())),
        }
    }
}

impl TryFrom<i8> for Part {
    type Error = ConversionError;

    fn try_from(c: i8) -> Result<Self, Self::Error> {
        c.try_into()
    }
}
