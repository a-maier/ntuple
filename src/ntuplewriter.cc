extern "C" {
#include "ntuplewriter.h"
}

#include <cassert>
#include <memory>
#include <mutex>

#include "TFile.h"
#include "TTree.h"

#include "root_interface.hh"

using namespace ntuple;

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
NTupleWriterCreateResult ntuple_create_writer(char const *file, char const *title) {
  try {
    std::lock_guard<std::mutex> lock{file_mutex};
    auto *writer = new NTupleWriter{
      {file, "RECREATE"},
      RootEvent{},
      nullptr
    };
    if(!writer || !writer->file.IsOpen()) {
      return NTupleWriterCreateResult {
        nullptr,
        OPEN_FAILED
      };
    }
    writer->file.cd();
    writer->tree = new TTree{"BHSntuples", title};
    if(!writer->tree) {
      return NTupleWriterCreateResult {
        nullptr,
        NO_TTREE,
      };
    }
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
    tree.Branch("alphasPower", &ev.alphas_power, "alphasPower/S");

    return NTupleWriterCreateResult {
      writer,
      NONE
    };
  } catch (...) {
    return NTupleWriterCreateResult {
      nullptr,
      EXCEPTION
    };
  }
}

void ntuple_delete_writer(NTupleWriter * writer) {
  assert(writer);
  assert(writer->tree);

  try {
    std::lock_guard<std::mutex> lock{file_mutex};
    writer->file.cd();
    writer->file.Write();
    writer->file.Close();
  } catch(...) {
    // let no exception escape,
    // it's undefined behaviour to let it propagate to rust
  }
  // destructors must not throw in any case
  delete writer;
}

NTupleWriteResult ntuple_write_event(NTupleWriter * writer, NTupleEvent const * event) {
  assert(writer);
  assert(event);
  assert(writer->tree);
  auto & ev = writer->event;

  if(event->nparticle < 0) return WRITE_NEGATIVE_NUMBER_OF_PARTICLES;
  if(event->nuwgt < 0) return WRITE_NEGATIVE_NUMBER_OF_WEIGHTS;
  if(static_cast<uint32_t>(event->nparticle) > MAX_NPARTICLE) {
    return WRITE_TOO_MANY_PARTICLES;
  }
  if(static_cast<uint32_t>(event->nuwgt) > MAX_NWGT) {
    return WRITE_TOO_MANY_WEIGHTS;
  }

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
  ev.alphas_power = event->alphas_power;

  try {
    // filling data into the tree may trigger a write,
    // so we have to lock the mutex and fix the current directory
    std::lock_guard<std::mutex> lock{file_mutex};
    writer->file.cd();
    writer->tree->Fill();
  } catch(...) {
    return WRITE_FILL_ERROR;
  }
  return WRITE_OK;
}
}
