import { EpochDays, Occurrence, Skull, Response, Quick, ProtoOccurrence } from './model';
import { sealed } from './context';

import { useContext, useEffect, useMemo, useState } from 'react';

export function useStore() {
  return useContext(sealed.StoreContext)!;
}

export const useSocketState = () => {
  const socket = useStore().getSocket();
  const [state, setState] = useState(socket.getState());

  useEffect(() => {
    const listener = socket.registerStateListener(setState);

    return () => {
      socket.unregisterStateListener(listener);
    }
  }, [socket]);

  return state;
};

export const useSkulls = () => {
  const store = useStore();
  const [skulls, setSkulls] = useState<Response<Skull>>(
    {
      items: store.getSkulls(),
      pending: !store.isSkullsLoaded(),
    }
  );

  useEffect(() => {
    const listener = store.registerSkullListener(s => setSkulls(
      {
        items: s,
        pending: !store.isSkullsLoaded(),
      }
    ));
    return () => store.removeSkullListener(listener);
  }, [store]);

  if (!skulls.error) {
    store
      .ensureSkulls()
      .catch(e => setSkulls({ ...skulls, error: e }));
  }

  return skulls;
};

export const useQuicks = () => {
  const store = useStore();
  const [quicks, setQuicks] = useState<Response<Quick>>(
    {
      items: store.getQuicks(),
      pending: !store.isQuicksLoaded(),
    }
  );

  useEffect(() => {
    const listener = store.registerQuickListener(q => setQuicks(
      {
        items: q,
        pending: !store.isQuicksLoaded(),
      }
    ));
    return () => store.removeQuickListener(listener);
  }, [store]);

  if (!quicks.error) {
    store
      .ensureQuicks()
      .catch(e => setQuicks({ ...quicks, error: e }));
  }

  return quicks;
};

export const useOccurrences = (
  start: EpochDays | Date | number,
  filter?: (o: Occurrence) => boolean,
) => {
  const startMillis = getMillis(start);
  const startDay = useMemo(() => new EpochDays(startMillis), [startMillis]);

  const parsedFilter = useMemo(
    () => !filter
      ? (o: Occurrence) => o.millis.getTime() >= startMillis
      : (o: Occurrence) => o.millis.getTime() >= startMillis && filter(o),
    [startMillis, filter],
  );

  const store = useStore();
  const [occurrences, setOccurrences] = useState<Response<Occurrence>>(
    {
      items: store.getOccurrences().filter(parsedFilter),
      pending: !store.isOccurrencesLoadedSince(startDay),
    }
  );

  // TODO: This is a bit of a hack to force update when start changes
  useEffect(() => {
    setOccurrences(
      {
        items: store.getOccurrences().filter(parsedFilter),
        pending: !store.isOccurrencesLoadedSince(startDay),
      }
    )
  }, [store, startDay, parsedFilter]);

  useEffect(() => {
    const listener = store.registerOccurrenceListener(o => setOccurrences(
      {
        items: o.filter(parsedFilter),
        pending: !store.isOccurrencesLoadedSince(startDay),
      }
    ));
    return () => store.removeOccurrenceListener(listener);
  }, [store, startDay, parsedFilter]);

  if (!occurrences.error) {
    store
      .ensureOccurrences(startDay)
      .catch(e => setOccurrences({ ...occurrences, error: e }));
  }

  return occurrences;
};

export const useEditOccurrence = () => {
  const store = useStore();
  const [pending, setPending] = useState(false);
  const [error, setError] = useState();

  const create = (occurrence: ProtoOccurrence) => {
    setPending(true);
    return store.edit.create(occurrence)
      .then(() => setPending(false))
      .catch(setError);
  };

  const update = (occurrence: Occurrence) => {
    setPending(true);
    return store.edit.update(occurrence)
      .then(() => setPending(false))
      .catch(setError);
  };

  const remove = (occurrence: Occurrence) => {
    setPending(true);
    return store.edit.remove(occurrence)
      .then(() => setPending(false))
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
  (time instanceof EpochDays)
    ? time.getMillis()
    : (time instanceof Date)
      ? time.getTime()
      : time;
