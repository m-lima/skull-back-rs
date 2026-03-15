import { Store } from './store';
import { sealed } from './hooks';

import { PropsWithChildren } from 'react';

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
