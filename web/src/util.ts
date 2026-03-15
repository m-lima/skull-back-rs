const useTls = import.meta.env.VITE_URL_TLS === 'true' ? 's' : '';
const host = import.meta.env.VITE_URL_HOST ?? 'localhost:3333';
const auth = import.meta.env.VITE_URL_AUTH ?? host;

export const query = {
  skulls: 'skulls',
  quicks: 'quicks',
  occurrences: 'occurrences',
};

export const path = {
  grid: '/',
  chart: '/chart',
  summary: '/summary',
} as const;

export const url = {
  ws: {
    binary: `ws${useTls}://${host}/ws/binary`,
    check: `http${useTls}://${host}/ws/binary`,
  },

  access: {
    login: `http${useTls}://${auth}/logout?redirect=http${useTls}://${auth}/login?redirect=${window.location.href}`,
    reset: `http${useTls}://${auth}/logout?redirect=http${useTls}://${auth}/reset?redirect=${window.location.href}`,
  },
} as const;
