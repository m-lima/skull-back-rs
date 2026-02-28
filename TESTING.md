# Behind nginx on nix

## Backend
```bash
$ cargo r -p server -- -U <user> -c -vvv -s 8558 <location>
```

## Frontend
```bash
$ PORT=8855 WDS_SOCKET_PATH='/ws' WDS_SOCKET_PORT='0' REACT_APP_URL_HOST='<host>' REACT_APP_URL_TLS='true' REACT_APP_URL_AUTH='<auth_host>' yarn start
```

## Nginx
```nginx

nginx = {
      virtualHosts = {
        "<host>" = {
          forceSSL = true;
          enableACME = true;
          http2 = true;
          http3 = true;
          extraConfig = ''
            endgame on;
          '';

          locations = {
            "/" = {
              proxyPass = "http://localhost:8855/";
              recommendedProxySettings = true;
              proxyWebsockets = true;
            };
            "/api/" = {
              proxyPass = "http://localhost:8558/api/";
              recommendedProxySettings = true;
            };
            "/ws/binary" = {
              proxyPass = "http://localhost:8558/ws/binary";
              recommendedProxySettings = true;
              proxyWebsockets = true;
            };
            "/ws/text" = {
              proxyPass = "http://localhost:8558/ws/text";
              recommendedProxySettings = true;
              proxyWebsockets = true;
            };
          };
        };
    };
}
```
