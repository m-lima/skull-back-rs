import * as util from './util';
import { Banner, Footer, Spinner } from './components/mod';
import { Grid, Summary } from './routes/mod';
import { SocketState } from './socket';
import { useSocketState } from './store/mod';

import './app.css';

import { BrowserRouter as Router, Navigate, Route, Routes } from 'react-router-dom';

const routes = [
  {
    path: util.path.grid,
    title: 'Quick values',
    icon: 'fas fa-th-large',
    element: <Grid />,
  },
  {
    path: util.path.summary,
    title: 'Summary',
    icon: 'fas fa-th-list',
    element: <Summary />,
  },
];

const App = () => {
  const socketState = useSocketState();

  switch (socketState) {
    case SocketState.Closed:
      return <Banner.Diconnected />;
    case SocketState.Error:
      return <Banner.SocketError />;
    case SocketState.Unauthorized:
      return <Banner.Unauthorized />;
  }

  return (
    <Router>
      {socketState === SocketState.Connecting && <div className='skull-connecting'><Spinner className='skull-connecting-spinner' margin />Connecting..</div>}
      <div className='skull'>
        <Routes>
          {routes.map((route, key) => (
            <Route
              key={key}
              path={route.path}
              element={route.element}
            />)
          )}
          <Route
            path='/*'
            element=<Navigate to={routes[0].path} />
          />
        </Routes>
      </div>
      <Footer routes={routes} />
    </Router>
  );
};

export default App;
