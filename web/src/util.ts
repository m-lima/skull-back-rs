export namespace query {
  export const skulls = 'skulls';
  export const quicks = 'quicks';
  export const occurrences = 'occurrences';
}

export namespace path {
  export const grid = '/';
  export const chart = '/chart';
  export const summary = '/summary';
}

export namespace url {
  const useTls = import.meta.env.VITE_URL_TLS === 'true' ? 's' : '';
  const host = import.meta.env.VITE_URL_HOST ?? 'localhost:3333';

  export namespace ws {
    export const binary = `ws${useTls}://${host}/ws/binary`;
    export const check = `http${useTls}://${host}/ws/binary`;
  }

  export namespace access {
    const auth = import.meta.env.VITE_URL_AUTH ?? host;
    export const login = `http${useTls}://${auth}/logout?redirect=http${useTls}://${auth}/login?redirect=${window.location}`;
    export const reset = `http${useTls}://${auth}/logout?redirect=http${useTls}://${auth}/reset?redirect=${window.location}`;
  }
}
