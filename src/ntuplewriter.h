#include <cstdint>
#include <stdint.h>

typedef struct NTupleWriter NTupleWriter;

typedef struct {
  int32_t id;
  int32_t nparticle;
  float* px;
  float* py;
  float* pz;
  float* E;
  double alphas;
  int32_t* kf;
  double weight;
  double weight2;
  double me_wgt;
  double me_wgt2;
  double x1;
  double x2;
  double x1p;
  double x2p;
  int32_t id1;
  int32_t id2;
  double fac_scale;
  double ren_scale;
  int32_t nuwgt;
  double* usr_wgts;
  unsigned char part;
  int16_t alphasPower;
} NTupleEvent;

typedef enum {
  OK,
  TOO_MANY_PARTICLES,
  TOO_MANY_WEIGHTS,
  NEGATIVE_NUMBER_OF_PARTICLES,
  NEGATIVE_NUMBER_OF_WEIGHTS
} WriteResult;

NTupleWriter *ntuple_create_writer(char const *file, char const *title);
void ntuple_delete_writer(NTupleWriter *);

WriteResult ntuple_write_event(NTupleWriter * writer, NTupleEvent const * event);
