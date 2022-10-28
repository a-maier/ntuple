use hepmc2::event::{EnergyUnit, LengthUnit, Particle, PdfInfo, Vertex};

use crate::Event;

const OUTGOING_STATUS: i32 = 1;

impl From<&Event> for hepmc2::Event {
    fn from(ev: &Event) -> Self {
        let nparticles = ev.nparticle as usize;
        let mut particles = Vec::with_capacity(nparticles as usize);
        for i in 0..nparticles {
            let p = [
                ev.energy[i] as f64,
                ev.px[i] as f64,
                ev.py[i] as f64,
                ev.pz[i] as f64,
            ];
            let p = Particle {
                id: ev.pdg_code[i],
                p: hepmc2::event::FourVector(p),
                m: 0.,
                theta: theta(p),
                phi: phi(p),
                status: OUTGOING_STATUS,
                ..Default::default()
            };
            particles.push(p)
        }
        let pdf_info = PdfInfo {
            parton_id: [ev.id1, ev.id2],
            x: [ev.x1, ev.x2],
            scale: ev.fac_scale,
            ..Default::default() // TODO: xf?
        };
        let vertices = vec![Vertex {
            particles_out: particles,
            // the exact `barcode` does not matter much,
            // but it must be different from the `end_vtx`
            // in the `particles` such that they are considered
            // outgoing with respect to the vertex
            // we choose a number that fits in a single ASCII byte to
            // not waste space
            barcode: 1,
            ..Default::default()
        }];
        let mut weights =
            vec![ev.weight, ev.weight2, ev.me_weight, ev.me_weight2];
        weights.extend_from_slice(&ev.user_weights);
        let weight_names = ["", "2", "ME", "ME2"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        hepmc2::Event {
            number: ev.id,
            scale: ev.ren_scale,
            weights,
            weight_names,
            vertices,
            pdf_info,
            energy_unit: EnergyUnit::GEV,
            length_unit: LengthUnit::MM,
            ..Default::default()
        }
    }
}

impl From<&hepmc2::Event> for Event {
    fn from(ev: &hepmc2::Event) -> Self {
        let outgoing = ev.vertices.iter().flat_map(|vx| {
            vx.particles_out
                .iter()
                .filter(|p| p.status == OUTGOING_STATUS)
        });

        let mut weight_names = ev.weight_names.clone();
        let mut weights = ev.weights.clone();
        let me_weight2 =
            if let Some(pos) = weight_names.iter().position(|n| n == "ME2") {
                weight_names.remove(pos);
                weights.remove(pos)
            } else {
                0.
            };
        let me_weight =
            if let Some(pos) = weight_names.iter().position(|n| n == "ME") {
                weight_names.remove(pos);
                weights.remove(pos)
            } else {
                0.
            };
        let weight2 =
            if let Some(pos) = weight_names.iter().position(|n| n == "2") {
                weight_names.remove(pos);
                weights.remove(pos)
            } else {
                0.
            };
        let weight = if !weights.is_empty() {
            weights.remove(0)
        } else {
            0.
        };

        Self {
            id: ev.number,
            nparticle: outgoing.clone().count() as i32,
            px: outgoing.clone().map(|p| p.p[1] as f32).collect(),
            py: outgoing.clone().map(|p| p.p[2] as f32).collect(),
            pz: outgoing.clone().map(|p| p.p[3] as f32).collect(),
            energy: outgoing.clone().map(|p| p.p[0] as f32).collect(),
            alphas: ev.alpha_qcd,
            pdg_code: outgoing.map(|p| p.id).collect(),
            weight,
            weight2,
            me_weight,
            me_weight2,
            x1: ev.pdf_info.x[0],
            x2: ev.pdf_info.x[1],
            id1: ev.pdf_info.parton_id[0],
            id2: ev.pdf_info.parton_id[1],
            fac_scale: ev.pdf_info.scale,
            ren_scale: ev.scale,
            user_weights: weights,
            ..Default::default()
        }
    }
}

fn phi(p: [f64; 4]) -> f64 {
    p[1].atan2(p[2])
}

fn theta(p: [f64; 4]) -> f64 {
    pt(p).atan2(p[3])
}

fn pt2(p: [f64; 4]) -> f64 {
    p[1] * p[1] + p[2] * p[2]
}

fn pt(p: [f64; 4]) -> f64 {
    pt2(p).sqrt()
}
