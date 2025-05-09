#ifndef NTUPLEWRITER_H
#define NTUPLEWRITER_H

#include "ntupleevent.h"
#include "create_error.h"

typedef struct NTupleWriter NTupleWriter;

typedef enum {
  WRITE_OK,
  WRITE_TOO_MANY_PARTICLES,
  WRITE_TOO_MANY_WEIGHTS,
  WRITE_NEGATIVE_NUMBER_OF_PARTICLES,
  WRITE_NEGATIVE_NUMBER_OF_WEIGHTS,
  WRITE_FILL_ERROR,
} NTupleWriteResult;

typedef struct {
  NTupleWriter* writer;
  NTupleCreateError error;
} NTupleWriterCreateResult;

NTupleWriterCreateResult ntuple_create_writer(char const *file, char const *title);
void ntuple_delete_writer(NTupleWriter *);

NTupleWriteResult ntuple_write_event(NTupleWriter * writer, NTupleEvent const * event);

#endif /* NTUPLEWRITER_H */
