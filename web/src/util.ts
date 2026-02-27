export namespace query {
  export const skulls = 'skulls';
  export const quicks = 'quicks';
  export const occurrences = 'occurrences';
}

export namespace path {
  export const grid = '/';
  export const summary = '/summary';
  export const chart: string | undefined = undefined;
}

export namespace url {
  const host = 'localhost:3333';

  export namespace ws {
    const protocol = 'ws';
    const apiPath = '';
    export const binary = `${protocol}://${host}${apiPath}/ws/binary`;
  }

  export namespace access {
    const protocol = 'http';
    export const check = `${protocol}://${host}/login`;
    export const logout = `${protocol}://${host}/logout?redirect=${protocol}://${host}/login?redirect=${window.location}`;
  }
}
