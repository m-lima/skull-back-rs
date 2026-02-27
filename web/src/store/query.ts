import { ErrorMessage } from './error';
import { sealed as modelSealed, Occurrence, ProtoOccurrence, RawQuick, Skull, EpochDays } from './model';
import { Socket } from '../socket';

export namespace sealed {
  export const getSkulls = (socket: Socket): Promise<Skull[]> => {
    const id = newRequestId();
    return socket.request({ id, skull: 'list' }, (message: any) => {
      const response = validateMessage(message, id, 'skulls');

      if (!!response) {
        return response.map(modelSealed.makeSkull);
      }
    });
  };

  export const getQuicks = (socket: Socket): Promise<RawQuick[]> => {
    const id = newRequestId();
    return socket.request({ id, quick: 'list' }, (message: any) => {
      const response = validateMessage(message, id, 'quicks');

      if (!!response) {
        return response.map(modelSealed.makeRawQuick);
      }
    });
  };

  export const getOccurrences = (socket: Socket, start: EpochDays, end?: EpochDays): Promise<Occurrence[]> => {
    const id = newRequestId();
    return socket.request(
      {
        id,
        occurrence: {
          search: {
            start: start.getMillis(),
            end: end?.getMillis(),
          }
        }
      },
      (message: any) => {
        const response = validateMessage(message, id, 'occurrences');

        if (!!response) {
          return response.map(modelSealed.makeOccurrence);
        }
      }
    );
  };

  export namespace edit {
    export const create = (socket: Socket, occurrence: ProtoOccurrence) => {
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
        (message: any) => {
          if (validateEditMessage(message, id, 'created')) {
            return true;
          }
        },
      );
    };

    export const update = (socket: Socket, occurrence: Occurrence) => {
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
        (message: any) => {
          if (validateEditMessage(message, id, 'updated')) {
            return true;
          }
        },
      );
    };

    export const remove = (socket: Socket, occurrence: Occurrence) => {
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
        (message: any) => {
          if (validateEditMessage(message, id, 'deleted')) {
            return true;
          }
        },
      );
    };

    const validateEditMessage = (message: any, id: number, action: string) => {
      const response = validateMessage(message, id, 'change');

      if (!!response) {
        if (response === action) {
          return true;
        } else {
          throw new ErrorMessage(
            {
              kind: 'invalidresponse',
              message: `Expected "${action}", got ${response}`,
            }
          );
        }
      }
    }
  }
}

const newRequestId = () => Math.floor(Math.random() * 1024 * 1024);

const validateMessage = (message: any, id: number, field: string) => {
  if (!('response' in message)) {
    return;
  }

  const response = message.response;
  if ('id' in response && response.id === id) {
    if ('error' in response) {
      throw new ErrorMessage(response.error);
    }

    if (field in response) {
      return response[field];
    } else {
      throw new ErrorMessage({ kind: 'invalidresponse', message: `Got: ${JSON.stringify(response)}` });
    }
  }
}
