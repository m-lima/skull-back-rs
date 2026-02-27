import * as util from '../util';
import { Icon } from './icon';

import './footer.css';

import { Link, useLocation } from 'react-router-dom';

interface Route {
  path: string,
  title: string,
  icon: string,
}

interface FooterProps {
  routes: Route[],
}

export const Footer = (props: FooterProps) => {
  const path = useLocation().pathname;

  return (
    <div className='footer'>
      <div className='footer-container'>
        {props.routes.map((route, key) => (
          <Link
            className='icon-button'
            key={key}
            to={route.path}
            title={route.title}
            id={path === route.path ? 'selected' : undefined}
          >
            <Icon icon={route.icon} />
          </Link>
        ))}
        {
          util.path.chart &&
            <Link
              className='icon-button'
              to={util.path.chart}
              title='Chart'
            >
              <Icon icon='fas fa-chart-line' />
            </Link>
        }
      </div>
    </div>
  );
};
