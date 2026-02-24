use crate::{server, utils};

#[derive(Debug, Clone)]
pub struct Client {
    client: reqwest::Client,
    uri: std::sync::Arc<String>,
}

impl From<&server::Server> for Client {
    fn from(server: &server::Server) -> Self {
        Self {
            client: reqwest::Client::new(),
            uri: server.uri(),
        }
    }
}

impl Client {
    pub async fn get(&self, path_and_query: impl AsRef<str>) -> reqwest::Response {
        let request = self.get_request(path_and_query);
        self.client.execute(request).await.unwrap()
    }

    pub async fn get_with(
        &self,
        path_and_query: impl AsRef<str>,
        f: impl Fn(&mut reqwest::Request),
    ) -> reqwest::Response {
        let mut request = self.get_request(path_and_query);
        f(&mut request);
        self.client.execute(request).await.unwrap()
    }

    pub async fn post(
        &self,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::body::Bytes>,
    ) -> reqwest::Response {
        let request = self.post_request(path_and_query, body);
        self.client.execute(request).await.unwrap()
    }

    pub async fn patch(
        &self,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::body::Bytes>,
    ) -> reqwest::Response {
        let request = self.patch_request(path_and_query, body);
        self.client.execute(request).await.unwrap()
    }

    pub async fn patch_with(
        &self,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::body::Bytes>,
        f: impl Fn(&mut reqwest::Request),
    ) -> reqwest::Response {
        let mut request = self.patch_request(path_and_query, body);
        f(&mut request);
        self.client.execute(request).await.unwrap()
    }

    pub async fn delete(
        &self,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::body::Bytes>,
    ) -> reqwest::Response {
        let request = self.delete_request(path_and_query, body);
        self.client.execute(request).await.unwrap()
    }

    pub async fn delete_with(
        &self,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::body::Bytes>,
        f: impl Fn(&mut reqwest::Request),
    ) -> reqwest::Response {
        let mut request = self.delete_request(path_and_query, body);
        f(&mut request);
        self.client.execute(request).await.unwrap()
    }
}

impl Client {
    fn request(
        &self,
        method: hyper::Method,
        path_and_query: impl AsRef<str>,
    ) -> reqwest::RequestBuilder {
        let uri = format!(
            "http://{authority}/{path_and_query}",
            authority = self.uri.as_str(),
            path_and_query = path_and_query.as_ref(),
        );

        self.client
            .request(method, uri)
            .header(utils::USER_HEADER, utils::USER)
    }

    fn get_request(&self, path_and_query: impl AsRef<str>) -> reqwest::Request {
        self.request(hyper::Method::GET, path_and_query)
            .build()
            .unwrap()
    }

    fn post_request(
        &self,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::body::Bytes>,
    ) -> reqwest::Request {
        self.request(hyper::Method::POST, path_and_query)
            .header(
                hyper::header::CONTENT_TYPE,
                hyper::header::HeaderValue::from_static("application/json"),
            )
            .body(body.into())
            .build()
            .unwrap()
    }

    fn patch_request(
        &self,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::body::Bytes>,
    ) -> reqwest::Request {
        self.request(hyper::Method::PATCH, path_and_query)
            .header(
                hyper::header::CONTENT_TYPE,
                hyper::header::HeaderValue::from_static("application/json"),
            )
            .body(body.into())
            .build()
            .unwrap()
    }

    fn delete_request(
        &self,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::body::Bytes>,
    ) -> reqwest::Request {
        self.request(hyper::Method::DELETE, path_and_query)
            .header(
                hyper::header::CONTENT_TYPE,
                hyper::header::HeaderValue::from_static("application/json"),
            )
            .body(body.into())
            .build()
            .unwrap()
    }
}
