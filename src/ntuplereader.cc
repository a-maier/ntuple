extern "C" {
#include "ntuplereader.h"
}

#include <mutex>

#include "TFile.h"
#include "TTree.h"

#include "root_interface.hh"

using namespace ntuple;

struct NTupleReader {
  TFile file;
  RootEvent event;
  TTree* tree;
  bool legacy_fmt;
};

extern "C" {
NTupleReaderCreateResult ntuple_create_reader(char const *file) {
  try {
    std::lock_guard<std::mutex> lock{file_mutex};
    auto *reader = new NTupleReader{
      {file, "READ"},
      RootEvent{},
      nullptr,
      false
    };
    if(!reader || !reader->file.IsOpen()) {
      return NTupleReaderCreateResult {
        nullptr,
        OPEN_FAILED
      };
    }
    reader->tree = dynamic_cast<TTree*>(reader->file.Get("BHSntuples"));
    if(!reader->tree) {
      // fallback: sometimes the name is a bit different
      reader->tree = dynamic_cast<TTree*>(reader->file.Get("t3"));
      reader->legacy_fmt = true;
    }
    if(!reader->tree) {
      return NTupleReaderCreateResult {
        nullptr,
        NO_TTREE
      };
    }

    reader->event.part.back() = '\0'; // ensure c string is null terminated

    auto & ev = reader->event;
    auto & tree = *reader->tree;
    tree.SetBranchAddress("id", &ev.id);
    tree.SetBranchAddress("nparticle", &ev.nparticle);
    tree.SetBranchAddress("px", ev.px.data());
    tree.SetBranchAddress("py", ev.py.data());
    tree.SetBranchAddress("pz", ev.pz.data());
    tree.SetBranchAddress("E", ev.E.data());
    tree.SetBranchAddress("alphas", &ev.alphas);
    tree.SetBranchAddress("kf", ev.kf.data());
    tree.SetBranchAddress("weight", &ev.weight);
    tree.SetBranchAddress("weight2", &ev.weight2);
    tree.SetBranchAddress("me_wgt", &ev.me_wgt);
    tree.SetBranchAddress("me_wgt2", &ev.me_wgt2);
    tree.SetBranchAddress("x1", &ev.x1);
    tree.SetBranchAddress("x2", &ev.x2);
    tree.SetBranchAddress("x1p", &ev.x1p);
    tree.SetBranchAddress("x2p", &ev.x2p);
    tree.SetBranchAddress("id1", &ev.id1);
    tree.SetBranchAddress("id2", &ev.id2);
    tree.SetBranchAddress("fac_scale", &ev.fac_scale);
    tree.SetBranchAddress("ren_scale", &ev.ren_scale);
    tree.SetBranchAddress("nuwgt", &ev.nuwgt);
    tree.SetBranchAddress("usr_wgts", ev.usr_wgts.data());
    tree.SetBranchAddress("part", &ev.part);
    if(reader->legacy_fmt) {
      tree.SetBranchAddress("alphasPower", &ev.alphas_power_char);
    } else {
      tree.SetBranchAddress("alphasPower", &ev.alphas_power);
    }

    return NTupleReaderCreateResult {
      reader,
      NONE
    };
  } catch (...) {
    return NTupleReaderCreateResult {
      nullptr,
      EXCEPTION
    };
  }
}

void ntuple_delete_reader(NTupleReader * reader) {
  assert(reader);
  assert(reader->tree);

  try {
    std::lock_guard<std::mutex> lock{file_mutex};
    reader->file.cd();
    reader->file.Close();
  } catch(...) {
    // let no exception escape,
    // it's undefined behaviour to let it propagate to rust
  }
  // destructors must not throw in any case
  delete reader;
}

int64_t ntuple_num_events(NTupleReader * reader) {
  assert(reader);
  assert(reader->tree);

  try {
    std::lock_guard<std::mutex> lock{file_mutex};
    reader->file.cd();
    return reader->tree->GetEntries();
  } catch(...) {
    return -1;
  }
}

NTupleReadResult ntuple_read_event(NTupleReader * reader, int64_t const idx) {
  assert(reader);
  assert(reader->tree);
  auto const & ev = reader->event;

  NTupleReadResult result;
  auto & event = result.event;
  auto & status = result.status;
  status = READ_OK;

  try {
    std::lock_guard<std::mutex> lock{file_mutex};
    reader->file.cd();

    const auto read = reader->tree->GetEntry(idx);
    switch(read) {
    case -1:
      status = READ_ERROR;
      return result;
    case 0:
      status = READ_NO_ENTRY;
      return result;
    default:;
    }

    event.id = ev.id;
    event.nparticle = ev.nparticle;
    event.px = ev.px.data();
    event.py = ev.py.data();
    event.pz = ev.pz.data();
    event.energy = ev.E.data();
    event.alphas = ev.alphas;
    event.kf = ev.kf.data();
    event.weight = ev.weight;
    event.weight2 = ev.weight2;
    event.me_wgt = ev.me_wgt;
    event.me_wgt2 = ev.me_wgt2;
    event.x1 = ev.x1;
    event.x2 = ev.x2;
    event.x1p = ev.x1p;
    event.x2p = ev.x2p;
    event.id1 = ev.id1;
    event.id2 = ev.id2;
    event.fac_scale = ev.fac_scale;
    event.ren_scale = ev.ren_scale;
    event.nuwgt = ev.nuwgt;
    event.usr_wgts = ev.usr_wgts.data();
    event.part = ev.part[0];
    if(reader->legacy_fmt) {
      event.alphas_power = ev.alphas_power_char;
    } else {
      event.alphas_power = ev.alphas_power;
    }

    if(ev.nparticle < 0) {
      status = READ_NEGATIVE_NUMBER_OF_PARTICLES;
      return result;
    }
    const uint32_t nparticle = ev.nparticle;
    if(nparticle > MAX_NPARTICLE) {
      status = READ_TOO_MANY_PARTICLES;
      return result;
    }
    if(ev.nuwgt < 0) {
      status = READ_NEGATIVE_NUMBER_OF_WEIGHTS;
      return result;
    }
    const uint32_t nwgt = ev.nuwgt;
    if(nwgt > MAX_NWGT) {
      status = READ_TOO_MANY_WEIGHTS;
      return result;
    }

    return result;
  } catch(...) {
    status = READ_EXCEPTION;
    return result;
  }
}
}
