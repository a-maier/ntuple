#ifndef NTUPLE_CREATE_ERROR_H
#define NTUPLE_CREATE_ERROR_H

typedef enum {
  NONE,
  OPEN_FAILED,
  NO_TTREE,
  EXCEPTION
} NTupleCreateError;

#endif /* NTUPLE_CREATE_ERROR_H */
