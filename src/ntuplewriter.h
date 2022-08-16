#ifndef NTUPLEWRITER_H
#define NTUPLEWRITER_H

#include "ntupleevent.h"

typedef struct NTupleWriter NTupleWriter;

typedef enum {
  OK,
  TOO_MANY_PARTICLES,
  TOO_MANY_WEIGHTS,
  NEGATIVE_NUMBER_OF_PARTICLES,
  NEGATIVE_NUMBER_OF_WEIGHTS,
  FILL_ERROR,
} WriteResult;

NTupleWriter *ntuple_create_writer(char const *file, char const *title);
void ntuple_delete_writer(NTupleWriter *);

WriteResult ntuple_write_event(NTupleWriter * writer, NTupleEvent const * event);

#endif /* NTUPLEWRITER_H */
