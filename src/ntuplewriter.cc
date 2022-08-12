extern "C" {
#include "ntuplewriter.h"
}

#include <array>
#include <cassert>
#include <cstdint>
#include <memory>
#include <mutex>

#include "TFile.h"
#include "TTree.h"

namespace {
  // We need a mutex to ensure that we write to the correct file
  //
  // ROOT has the concept of a current directory, and when calling
  // `Write()` data are written to this directory. The current directory
  // changes whenever we open a new `TFile`, so we have to protect any
  // `Write()` with a mutex against `TFile` construction. What is worse,
  // we don't have control over when ROOT internally calls `Write()`.
  // So we err on the conservative side, i.e. we lock the mutex and fix
  // the current directory whenever we change data that might be written to file.
  std::mutex file_mutex;

  // the following is guaranteed by ROOT documentation,
  // so naturally we don't trust it
  static_assert(sizeof(Int_t) == sizeof(int32_t));
  static_assert(sizeof(Short_t) == sizeof(int16_t));
  // for floating-point types, cross fingers and hope for the best
  static_assert(sizeof(Float_t) == sizeof(float));
  static_assert(sizeof(Double_t) == sizeof(double));

  constexpr std::size_t MAX_NPARTICLE = 100;
  constexpr std::size_t MAX_NWGT = 100;

  struct RootEvent {
     Int_t id;
     Int_t nparticle;
     std::array<Float_t, MAX_NPARTICLE> px;
     std::array<Float_t, MAX_NPARTICLE> py;
     std::array<Float_t, MAX_NPARTICLE> pz;
     std::array<Float_t, MAX_NPARTICLE> E;
     Double_t alphas;
     std::array<Int_t, MAX_NPARTICLE> kf;
     Double_t weight;
     Double_t weight2;
     Double_t me_wgt;
     Double_t me_wgt2;
     Double_t x1;
     Double_t x2;
     Double_t x1p;
     Double_t x2p;
     Int_t id1;
     Int_t id2;
     Double_t fac_scale;
     Double_t ren_scale;
     Int_t nuwgt;
     std::array<Double_t, MAX_NWGT> usr_wgts;
     std::array<Char_t, 2> part;
     Short_t alphasPower;
   };
}

struct NTupleWriter {
  TFile file;
  RootEvent event;
  // we use a pointer here to work around a presumed ROOT bug,
  // where `tree` is not associated with `file`,
  // although `file` is guaranteed to be constructed first
  // a raw pointer is the correct type, since the file silently takes ownership
  TTree* tree;
};

extern "C" {
NTupleWriter *ntuple_create_writer(char const *file, char const *title) {
  try {
    std::scoped_lock lock{file_mutex};
    auto *writer = new NTupleWriter{
      TFile(file, "RECREATE"),
      RootEvent{},
      new TTree{"BHSntuples", title}
    };
    if(!writer) return nullptr;
    if(!writer->tree) return nullptr;
    if(!writer->file.IsOpen()) return nullptr;
    writer->event.part.back() = '\0'; // ensure c string is null terminated

    auto &ev = writer->event;
    auto &tree = *writer->tree;
    tree.Branch("id", &ev.id, "id/I");
    tree.Branch("nparticle", &ev.nparticle, "nparticle/I");
    tree.Branch("px", ev.px.data(), "px[nparticle]/F");
    tree.Branch("py", ev.py.data(), "py[nparticle]/F");
    tree.Branch("pz", ev.pz.data(), "pz[nparticle]/F");
    tree.Branch("E", ev.E.data(), "E[nparticle]/F");
    tree.Branch("alphas", &ev.alphas, "alphas/D");
    tree.Branch("kf", ev.kf.data(), "kf[nparticle]/I");
    tree.Branch("weight", &ev.weight, "weight/D");
    tree.Branch("weight2", &ev.weight2, "weight2/D");
    // intentional typo 'me_wtg' for compatibility with existing files
    tree.Branch("me_wgt", &ev.me_wgt, "me_wtg/D");
    tree.Branch("me_wgt2", &ev.me_wgt2, "me_wtg2/D");
    tree.Branch("x1", &ev.x1, "x1/D");
    tree.Branch("x2", &ev.x2, "x2/D");
    tree.Branch("x1p", &ev.x1p, "x1p/D");
    tree.Branch("x2p", &ev.x2p, "x2p/D");
    tree.Branch("id1", &ev.id1, "id1/I");
    tree.Branch("id2", &ev.id2, "id2/I");
    tree.Branch("fac_scale", &ev.fac_scale, "fac_scale/D");
    tree.Branch("ren_scale", &ev.ren_scale, "ren_scale/D");
    tree.Branch("nuwgt", &ev.nuwgt, "nuwgt/I");
    tree.Branch("usr_wgts", ev.usr_wgts.data(), "usr_wgts[nuwgt]/D");
    tree.Branch("part", ev.part.data(), "part/C");
    tree.Branch("alphasPower", &ev.alphasPower, "alphasPower/S");

    return writer;
  } catch (...) {
    return nullptr;
  }
}

void ntuple_delete_writer(NTupleWriter * writer) {
  assert(writer);
  assert(writer->tree);

  try {
    std::scoped_lock lock{file_mutex};
    writer->file.cd();
    writer->tree->Write();
    writer->file.Close();
  } catch(...) {
    // let no exception escape,
    // it's undefined behaviour to let it propagate to rust
  }
  // destructors must not throw in any case
  delete writer;
}

WriteResult ntuple_write_event(NTupleWriter * writer, NTupleEvent const * event) {
  assert(writer);
  assert(event);
  assert(writer->tree);
  auto & ev = writer->event;

  if(event->nparticle < 0) return NEGATIVE_NUMBER_OF_PARTICLES;
  if(event->nuwgt < 0) return NEGATIVE_NUMBER_OF_WEIGHTS;
  if(static_cast<uint32_t>(event->nparticle) > MAX_NPARTICLE) return TOO_MANY_PARTICLES;
  if(static_cast<uint32_t>(event->nuwgt) > MAX_NWGT) return TOO_MANY_WEIGHTS;

  ev.id = event->id;
  ev.nparticle = event->nparticle;
  std::copy(event->px, event->px + event->nparticle, ev.px.begin());
  std::copy(event->py, event->py + event->nparticle, ev.py.begin());
  std::copy(event->pz, event->pz + event->nparticle, ev.pz.begin());
  std::copy(event->energy, event->energy + event->nparticle, ev.E.begin());
  ev.alphas = event->alphas;
  std::copy(event->kf, event->kf + event->nparticle, ev.kf.begin());
  ev.weight = event->weight;
  ev.weight2 = event->weight2;
  ev.me_wgt = event->me_wgt;
  ev.me_wgt2 = event->me_wgt2;
  ev.x1 = event->x1;
  ev.x2 = event->x2;
  ev.x1p = event->x1p;
  ev.x2p = event->x2p;
  ev.id1 = event->id1;
  ev.id2 = event->id2;
  ev.fac_scale = event->fac_scale;
  ev.ren_scale = event->ren_scale;
  ev.nuwgt = event->nuwgt;
  std::copy(event->usr_wgts, event->usr_wgts + event->nuwgt, ev.usr_wgts.begin());
  ev.part[0] = event->part;
  ev.alphasPower = event->alphas_power;

  try {
    // filling data into the tree may trigger a write,
    // so we have to lock the mutex and fix the current directory
    std::scoped_lock lock{file_mutex};
    writer->file.cd();
    writer->tree->Fill();
  } catch(...) {
    return FILL_ERROR;
  }
  return OK;
}
}
