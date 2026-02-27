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
  const host = process.env.REACT_APP_URL_HOST === undefined ? 'localhost:3333' : process.env.REACT_APP_URL_HOST;

  export namespace ws {
    const protocol = process.env.REACT_APP_URL_WS_PROTOCOL === undefined ? 'ws' : process.env.REACT_APP_URL_WS_PROTOCOL;
    export const binary = `${protocol}://${host}/ws/binary`;
  }

  export namespace access {
    const auth = process.env.REACT_APP_URL_AUTH === undefined ? host : process.env.REACT_APP_URL_AUTH;
    const protocol = process.env.REACT_APP_URL_ACCESS_PROTOCOL === undefined ? 'http' : process.env.REACT_APP_URL_ACCESS_PROTOCOL;
    export const check = `${protocol}://${auth}/login`;
    export const logout = `${protocol}://${auth}/logout?redirect=${protocol}://${host}/login?redirect=${window.location}`;
  }
}
