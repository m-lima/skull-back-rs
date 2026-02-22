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
    pub async fn get(
        &self,
        path_and_query: impl AsRef<str>,
    ) -> hyper::Response<hyper::body::Bytes> {
        let request = self.get_request(path_and_query);
        self.client.request(request).await.unwrap()
    }

    pub async fn get_with(
        &self,
        path_and_query: impl AsRef<str>,
        f: impl Fn(&mut hyper::Request<hyper::body::Bytes>),
    ) -> hyper::Response<hyper::body::Bytes> {
        let mut request = self.get_request(path_and_query);
        f(&mut request);
        self.client.request(request).await.unwrap()
    }

    pub async fn post(
        &self,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::body::Bytes>,
    ) -> hyper::Response<hyper::body::Bytes> {
        let request = self.post_request(path_and_query, body);
        self.client.request(request).await.unwrap()
    }

    pub async fn patch(
        &self,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::body::Bytes>,
    ) -> hyper::Response<hyper::body::Bytes> {
        let request = self.patch_request(path_and_query, body);
        self.client.request(request).await.unwrap()
    }

    pub async fn patch_with(
        &self,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::body::Bytes>,
        f: impl Fn(&mut hyper::Request<hyper::body::Bytes>),
    ) -> hyper::Response<hyper::body::Bytes> {
        let mut request = self.patch_request(path_and_query, body);
        f(&mut request);
        self.client.request(request).await.unwrap()
    }

    pub async fn delete(
        &self,
        path_and_query: impl AsRef<str>,
    ) -> hyper::Response<hyper::body::Bytes> {
        let request = self.delete_request(path_and_query);
        self.client.request(request).await.unwrap()
    }

    pub async fn delete_with(
        &self,
        path_and_query: impl AsRef<str>,
        f: impl Fn(&mut hyper::Request<hyper::body::Bytes>),
    ) -> hyper::Response<hyper::body::Bytes> {
        let mut request = self.delete_request(path_and_query);
        f(&mut request);
        self.client.request(request).await.unwrap()
    }
}

impl Client {
    fn request(
        &self,
        method: hyper::Method,
        path_and_query: impl AsRef<str>,
    ) -> reqwest::RequestBuilder {
        let uri = hyper::Uri::builder()
            .scheme("http")
            .authority(self.uri.as_str())
            .path_and_query(path_and_query.as_ref())
            .build()
            .unwrap();

        self.client
            .request(method, uri)
            .header(utils::USER_HEADER, utils::USER)
    }

    fn get_request(&self, path_and_query: impl AsRef<str>) -> hyper::Request<hyper::body::Bytes> {
        self.request(path_and_query)
            .method(hyper::Method::GET)
            .body(hyper::body::Bytes::new())
            .unwrap()
    }

    fn post_request(
        &self,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::body::Bytes>,
    ) -> hyper::Request<hyper::body::Bytes> {
        self.request(path_and_query)
            .method(hyper::Method::POST)
            .header(
                hyper::header::CONTENT_TYPE,
                hyper::header::HeaderValue::from_static("application/json"),
            )
            .body(body.into())
            .unwrap()
    }

    fn patch_request(
        &self,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::body::Bytes>,
    ) -> hyper::Request<hyper::body::Bytes> {
        self.request(path_and_query)
            .method(hyper::Method::PATCH)
            .body(body.into())
            .unwrap()
    }

    fn delete_request(
        &self,
        path_and_query: impl AsRef<str>,
    ) -> hyper::Request<hyper::body::Bytes> {
        self.request(path_and_query)
            .method(hyper::Method::DELETE)
            .body(hyper::body::Bytes::new())
            .unwrap()
    }
}
