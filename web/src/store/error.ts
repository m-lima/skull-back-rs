import { Timeout } from '../socket';

export enum ErrorKind {
  Timeout = 'Timeout',
  BadRequest = 'Bad Request',
  NotFound = 'Not Found',
  InternalError = 'Internal Error',
  InvalidResponse = 'Invalid Response',
  Unknown = 'Unknown Error',
}

export class ErrorMessage extends Error {
  readonly kind: ErrorKind;
  readonly message: string;

  public constructor(cause: unknown, message?: string) {
    super();

    if (typeof cause === 'string') {
      this.kind = parseKind(cause);
      this.message = parseMessage(message, this.kind);
    } else if (cause instanceof ErrorMessage) {
      this.kind = cause.kind;
      this.message = cause.message;
    } else if (cause instanceof Timeout) {
      this.kind = ErrorKind.Timeout;
      this.message = `Request timed out after ${cause.getMillis()}ms`;
    } else if (cause instanceof Array && cause.length < 3) {
      this.kind = parseKind(cause[0]);
      this.message = parseMessage(cause[1], this.kind);
    } else if (typeof cause === 'object' && cause !== null) {
      this.kind = parseKind('kind' in cause ? cause.kind : undefined);
      this.message = parseMessage('message' in cause ? cause.message : undefined, this.kind);
    } else {
      this.kind = ErrorKind.Unknown;
      this.message = ErrorKind.Unknown;
    }
  }
}

const parseKind = (kind: unknown): ErrorKind => {
  if (typeof kind !== 'string') {
    return ErrorKind.Unknown;
  }

  const simplified = kind.replaceAll(' ', '').toLowerCase();
  switch (simplified) {
    case 'timeout':
      return ErrorKind.Timeout;
    case 'badrequest':
      return ErrorKind.BadRequest;
    case 'notfound':
      return ErrorKind.NotFound;
    case 'internalerror':
      return ErrorKind.InternalError;
    case 'invalidresponse':
      return ErrorKind.InvalidResponse;
    default:
      return ErrorKind.Unknown;
  }
};

const parseMessage = (message: unknown, kind: ErrorKind): string => {
  if (message === 'string') {
    return message;
  }

  return kind;
};
