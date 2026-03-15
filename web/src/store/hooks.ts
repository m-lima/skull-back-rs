import { EpochDays, Occurrence, Skull, Response, Quick, ProtoOccurrence } from './model';
import {Store} from './store';

import { createContext, useContext, useEffect, useMemo, useState } from 'react';

export const sealed = {
  StoreContext: createContext<Store | undefined>(undefined),
} as const;

export function useStore() {
  const context = useContext(sealed.StoreContext);
  if (context === undefined) {
    throw new Error('`useStore` must be used inside a <StoreProvider>');
  }
  return context;
}

export const useSocketState = () => {
  const socket = useStore().getSocket();
  const [state, setState] = useState(socket.getState());

  useEffect(() => {
    const listener = socket.registerStateListener(setState);

    return () => {
      socket.unregisterStateListener(listener);
    };
  }, [socket]);

  return state;
};

export const useSkulls = () => {
  const store = useStore();
  const [skulls, setSkulls] = useState<Response<Skull>>({
    items: store.getSkulls(),
  });

  useEffect(() => {
    const listener = store.registerSkullListener(s => {
      setSkulls({
        items: s,
      });
    });
    return () => {
      store.removeSkullListener(listener);
    };
  }, [store]);

  if (!skulls.error) {
    store.ensureSkulls().catch((e: unknown) => {
      setSkulls({ ...skulls, error: e });
    });
  }

  return { ...skulls, pending: !store.isSkullsLoaded() };
};

export const useQuicks = () => {
  const store = useStore();
  const [quicks, setQuicks] = useState<Response<Quick>>({
    items: store.getQuicks(),
  });

  useEffect(() => {
    const listener = store.registerQuickListener(q => {
      setQuicks({
        items: q,
      });
    });
    return () => {
      store.removeQuickListener(listener);
    };
  }, [store]);

  if (!quicks.error) {
    store.ensureQuicks().catch((e: unknown) => {
      setQuicks({ ...quicks, error: e });
    });
  }

  return { ...quicks, pending: !store.isQuicksLoaded() };
};

export const useOccurrences = (
  startDay: EpochDays,
  filter?: (o: Occurrence) => boolean,
) => {
  const startMillis = getMillis(startDay);

  const parsedFilter = useMemo(
    () =>
      !filter
        ? (o: Occurrence) => o.millis.getTime() >= startMillis
        : (o: Occurrence) => o.millis.getTime() >= startMillis && filter(o),
    [startMillis, filter],
  );

  const store = useStore();
  const [occurrences, setOccurrences] = useState<Response<Occurrence>>({
    items: store.getOccurrences().filter(parsedFilter),
  });

  useEffect(() => {
    const listener = store.registerOccurrenceListener(o => {
      setOccurrences({
        items: o.filter(parsedFilter),
      });
    });
    return () => {
      store.removeOccurrenceListener(listener);
    };
  }, [store, startDay, parsedFilter]);

  if (!occurrences.error) {
    store.ensureOccurrences(startDay).catch((e: unknown) => {
      setOccurrences({ ...occurrences, error: e });
    });
  }

  return {...occurrences, pending: !store.isOccurrencesLoadedSince(startDay)};
};

export const useEditOccurrence = () => {
  const store = useStore();
  const [pending, setPending] = useState(false);
  const [error, setError] = useState();

  const create = (occurrence: ProtoOccurrence) => {
    setPending(true);
    return store.edit
      .create(occurrence)
      .then(() => {
        setPending(false);
      })
      .catch(setError);
  };

  const update = (occurrence: Occurrence) => {
    setPending(true);
    return store.edit
      .update(occurrence)
      .then(() => {
        setPending(false);
      })
      .catch(setError);
  };

  const remove = (occurrence: Occurrence) => {
    setPending(true);
    return store.edit
      .remove(occurrence)
      .then(() => {
        setPending(false);
      })
      .catch(setError);
  };

  return {
    create,
    update,
    remove,
    pending,
    error,
  };
};

const getMillis = (time: EpochDays | Date | number) =>
  time instanceof EpochDays ? time.getMillis() : time instanceof Date ? time.getTime() : time;
