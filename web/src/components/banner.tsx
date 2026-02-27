import * as util from '../util';
import { Icon, Spinner } from './icon';

import './banner.css';
import { ErrorKind, ErrorMessage } from '../store/mod';

export const Loading = () => (
  <div className='banner'>
    <Spinner />
    Loading..
  </div>
);

interface ErrorProps {
  error?: any,
}

export const Error = (props: ErrorProps) => {
  if (!props.error || !(props.error instanceof ErrorMessage)) {
    return (
      <div className='banner'>
        <Icon icon='fas fa-sad-tear' />
        Something went wrong..
        <Refresh />
      </div>
    );
  }

  const icon = props.error.kind === ErrorKind.Timeout
    ? 'fas fa-clock'
    : 'fas fa-sad-tear';

  return (
    <div className='banner'>
      <Icon icon={icon} />
      {props.error.kindString()}
      <code className='banner-error-message'>
        {props.error.message}
      </code>
      <Refresh />
    </div>
  );
};

export const SocketError = () => (
  <div className='banner'>
    <Icon icon='fas fa-unlink' />
    Connection error
    <Refresh />
  </div>
);

export const Diconnected = () => (
  <div className='banner'>
    <Icon icon='fas fa-plug' />
    The server has disconnected
    <Refresh />
  </div>
);

export const Unauthorized = () => (
  <div className='banner'>
    <Icon icon='fas fa-fingerprint' />
    Unauthorized
    <Refresh />
  </div>
);

export const Empty = () => (
  <div className='banner'>
    <Icon icon='fas fa-smile-wink' />
    No skulls found
  </div>
);

export const NoQuicks = () => (
  <div className='banner'>
    <Icon icon='fas fa-search' />
    No quicks found
  </div>
);

const Refresh = () =>
  <>
    <a className='banner-action' href={window.location.href}>
      <Icon margin icon='fas fa-sync' />
      Refresh
    </a>
    <a className='banner-action' href={util.url.access.logout}>
      <Icon margin icon='fas fa-sign-out-alt' />
      Logout
    </a>
  </>
