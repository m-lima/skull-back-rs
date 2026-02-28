import * as util from './util';
import App from './app';
import { Ribbon } from './components/mod';
import { Socket } from './socket';
import { Store, StoreProvider } from './store/mod';

import './index.css';

import React from 'react';
import ReactDOM from 'react-dom/client';

const socket = new Socket(util.url.ws.binary, util.url.ws.check);
const store = new Store(socket);

const root = ReactDOM.createRoot(document.getElementById('root') as HTMLElement);
root.render(
  <React.StrictMode>
    {process.env.NODE_ENV === 'development' && <Ribbon text='Development' />}
    <StoreProvider store={store}>
      <App />
    </StoreProvider>
  </React.StrictMode>
);
