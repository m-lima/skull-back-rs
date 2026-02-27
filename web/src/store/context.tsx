import { Store } from './store';

import { createContext, PropsWithChildren } from 'react';

export namespace sealed {
  export const StoreContext = createContext<Store | undefined>(undefined);
}

interface StoreProviderProps {
  store: Store,
}

export const StoreProvider = (props: PropsWithChildren<StoreProviderProps>) => {
  return (
    <sealed.StoreContext.Provider value={props.store} >
      {props.children}
    </sealed.StoreContext.Provider>
  );
}
