import { Store } from './store';

import { createContext, PropsWithChildren } from 'react';

export const sealed = {
  StoreContext: createContext<Store | undefined>(undefined),
} as const;

interface StoreProviderProps {
  store: Store;
}

export const StoreProvider = (props: PropsWithChildren<StoreProviderProps>) => {
  return (
    <sealed.StoreContext.Provider value={props.store}>
      {props.children}
    </sealed.StoreContext.Provider>
  );
};
