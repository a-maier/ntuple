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
    pub id2:  i32,
    pub fac_scale: f64,
    pub ren_scale: f64,
    pub user_weights: Vec<f64>,
    pub part: Part,
    pub alphas_power: i16,
}

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Part {
    #[default]
    B,
    I,
    R,
    V,
}

impl From<Part> for u8 {
    fn from(p: Part) -> Self {
        use Part::*;
        match p {
            B => b'B',
            I => b'I',
            R => b'R',
            V => b'V',
        }
    }
}

impl From<Part> for i8 {
    fn from(p: Part) -> Self {
        u8::from(p) as i8
    }
}
