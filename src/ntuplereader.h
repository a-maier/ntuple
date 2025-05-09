#ifndef NTUPLEREADER_H
#define NTUPLEREADER_H

#include "ntupleevent.h"
#include "create_error.h"

typedef struct NTupleReader NTupleReader;

typedef enum {
  READ_OK,
  READ_NO_ENTRY,
  READ_TOO_MANY_PARTICLES,
  READ_TOO_MANY_WEIGHTS,
  READ_NEGATIVE_NUMBER_OF_PARTICLES,
  READ_NEGATIVE_NUMBER_OF_WEIGHTS,
  READ_ERROR,
  READ_EXCEPTION,
} NTupleReadStatus;

typedef struct {
  NTupleEvent event;
  NTupleReadStatus status;
} NTupleReadResult;

typedef struct {
  NTupleReader* reader;
  NTupleCreateError error;
} NTupleReaderCreateResult;

NTupleReaderCreateResult ntuple_create_reader(char const* file);
void ntuple_delete_reader(NTupleReader* reader);

int64_t ntuple_num_events(NTupleReader* reader);
NTupleReadResult ntuple_read_event(NTupleReader* reader, int64_t idx);

#endif /* NTUPLEREADER_H */
