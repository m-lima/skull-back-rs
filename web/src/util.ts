export namespace query {
  export const skulls = 'skulls';
  export const quicks = 'quicks';
  export const occurrences = 'occurrences';
}

export namespace path {
  export const grid = '/';
  export const summary = '/summary';
  export const chart: string | undefined = process.env.REACT_APP_PATH_CHART;
}

export namespace url {
  const useTls =  process.env.REACT_APP_URL_TLS === 'true' ? 's' : '';
  const host = process.env.REACT_APP_URL_HOST === undefined ? 'localhost:3333' : process.env.REACT_APP_URL_HOST;

  export namespace ws {
    export const binary = `ws${useTls}://${host}/ws/binary`;
    export const check = `http${useTls}://${host}/ws/binary`;
  }

  export namespace access {
    const auth = process.env.REACT_APP_URL_AUTH === undefined ? host : process.env.REACT_APP_URL_AUTH;
    export const logout = `http${useTls}://${auth}/logout?redirect=http${useTls}://${host}/login?redirect=${window.location}`;
  }
}
