import { ErrorKind, ErrorMessage } from './error';
import {
  sealed as modelSealed,
  Occurrence,
  ProtoOccurrence,
  RawQuick,
  Skull,
  EpochDays,
} from './model';
import { Socket } from '../socket';

export const sealed = {
  getSkulls: (socket: Socket): Promise<Skull[]> => {
    const id = newRequestId();
    return socket.request({ id, skull: 'list' }, (message: unknown) => {
      const response = validateMessage(message, id, 'skulls', r => r instanceof Array);

      if (response !== undefined) {
        return response.map(s => {
          if (modelSealed.isSkullTuple(s)) {
            return modelSealed.makeSkull(s);
          }
          throw new ErrorMessage(ErrorKind.InvalidResponse, `Expected a skull tuple, got ${s}`);
        });
      }
    });
  },

  getQuicks: (socket: Socket): Promise<RawQuick[]> => {
    const id = newRequestId();
    return socket.request({ id, occurrence: 'quick' }, (message: unknown) => {
      const response = validateMessage(message, id, 'quicks', r => r instanceof Array);

      if (response !== undefined) {
        return response.map(q => {
          if (modelSealed.isQuickTuple(q)) {
            return modelSealed.makeRawQuick(q);
          }
          throw new ErrorMessage(ErrorKind.InvalidResponse, `Expected a quick tuple, got ${q}`);
        });
      }
    });
  },

  getOccurrences: (socket: Socket, start: EpochDays, end?: EpochDays): Promise<Occurrence[]> => {
    const id = newRequestId();
    return socket.request(
      {
        id,
        occurrence: {
          search: {
            start: start.getMillis(),
            end: end?.getMillis(),
          },
        },
      },
      (message: unknown) => {
        const response = validateMessage(message, id, 'occurrences', r => r instanceof Array);

        if (response !== undefined) {
          return response.map(o => {
            if (modelSealed.isOccurrenceTuple(o)) {
              return modelSealed.makeOccurrence(o);
            }
            throw new ErrorMessage(
              ErrorKind.InvalidResponse,
              `Expected an occurrence tuple, got ${o}`,
            );
          });
        }
      },
    );
  },

  edit: {
    create: (socket: Socket, occurrence: ProtoOccurrence) => {
      const id = newRequestId();
      return socket.request(
        {
          id,
          occurrence: {
            create: {
              items: [{ ...occurrence, millis: occurrence.millis.getTime() }],
            },
          },
        },
        (message: unknown) => {
          if (validateEditMessage(message, id, 'created')) {
            return true;
          }
        },
      );
    },

    update: (socket: Socket, occurrence: Occurrence) => {
      const id = newRequestId();
      return socket.request(
        {
          id,
          occurrence: {
            update: {
              id: occurrence.id,
              skull: { set: occurrence.skull },
              amount: { set: occurrence.amount },
              millis: { set: occurrence.millis.getTime() },
            },
          },
        },
        (message: unknown) => {
          if (validateEditMessage(message, id, 'updated')) {
            return true;
          }
        },
      );
    },

    remove: (socket: Socket, occurrence: Occurrence) => {
      const id = newRequestId();
      return socket.request(
        {
          id,
          occurrence: {
            delete: {
              id: occurrence.id,
            },
          },
        },
        (message: unknown) => {
          if (validateEditMessage(message, id, 'deleted')) {
            return true;
          }
        },
      );
    },
  },
} as const;

const newRequestId = () => Math.floor(Math.random() * 1024 * 1024);

const validateMessage = <R>(
  message: unknown,
  id: number,
  field: string,
  typeCheck: (r: unknown) => r is R,
): R | undefined => {
  if (typeof message !== 'object' || message === null || !('response' in message)) {
    return;
  }

  if (
    typeof message.response === 'object' &&
    message.response !== null &&
    'id' in message.response &&
    message.response.id === id
  ) {
    const response = message.response as Record<string, unknown>;
    if ('error' in response) {
      throw new ErrorMessage(response.error);
    }

    if (field in response && typeCheck(response[field])) {
      return response[field];
    } else {
      throw new ErrorMessage(ErrorKind.InvalidResponse, `Got: ${JSON.stringify(response)}`);
    }
  }
};

const validateEditMessage = (message: unknown, id: number, action: string) => {
  const response = validateMessage(message, id, 'change', r => typeof r === 'string');

  if (response !== undefined) {
    if (response === action) {
      return true;
    } else {
      throw new ErrorMessage(ErrorKind.InvalidResponse, `Expected "${action}", got ${response}`);
    }
  }
};
