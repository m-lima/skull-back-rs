import { Timeout } from '../socket';

export enum ErrorKind {
  Timeout,
  BadRequest,
  NotFound,
  InternalError,
  InvalidResponse,
  Unknown,
}

export class ErrorMessage {
  readonly kind: ErrorKind;
  readonly message: string;

  public constructor(error: any) {
    if (error instanceof ErrorMessage) {
      this.kind = error.kind;
      this.message = error.message;
      return;
    } else if (error instanceof Timeout) {
      this.kind = ErrorKind.Timeout;
      this.message = `Request timed out after ${error.getMillis()}ms`;
      return;
    }

    if (error instanceof Array && error.length < 3) {
      error = {
        kind: error[0],
        message: error[1],
      };
    }

    this.kind = parseKind(error.kind);

    if (!!error['message'] && typeof (error.message) === 'string') {
      this.message = error.message;
    } else {
      this.message = kindToString(this.kind);
    }
  }

  public kindString() {
    return kindToString(this.kind);
  }
}

const parseKind = (kind?: string): ErrorKind => {
  if (!kind) {
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
}

const kindToString = (kind: ErrorKind) => {
  switch (kind) {
    case ErrorKind.Timeout:
      return 'Timeout';
    case ErrorKind.BadRequest:
      return 'Bad Request';
    case ErrorKind.NotFound:
      return 'Not Found';
    case ErrorKind.InternalError:
      return 'Internal Error';
    case ErrorKind.InvalidResponse:
      return 'Invalid Response';
    default:
      return 'Unknown Error';
  }
}
