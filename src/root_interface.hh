#ifndef ROOT_INTERFACE_H
#define ROOT_INTERFACE_H

#include <array>
#include <cstddef>
#include <stddef.h>
#include <mutex>

#include "RtypesCore.h"

namespace ntuple {
  // We need a mutex to ensure that we write to the correct file
  //
  // ROOT has the concept of a current directory, and when calling
  // `Write()` data are written to this directory. The current directory
  // changes whenever we open a new `TFile`, so we have to protect any
  // `Write()` with a mutex against `TFile` construction. What is worse,
  // we don't have control over when ROOT internally calls `Write()`.
  // So we err on the conservative side, i.e. we lock the mutex and fix
  // the current directory whenever we change data that might be written to file.
  extern std::mutex file_mutex;

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

} // namespace ntuple
#endif /* ROOT_INTERFACE_H */
