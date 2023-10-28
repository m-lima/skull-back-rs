use crate::{server, utils};

#[derive(Debug, Clone)]
pub struct Client {
    client: hyper::Client<hyper::client::HttpConnector>,
    uri: std::sync::Arc<String>,
}

impl From<&server::Server> for Client {
    fn from(server: &server::Server) -> Self {
        Self {
            client: hyper::Client::new(),
            uri: server.uri(),
        }
    }
}

impl Client {
    pub async fn get(&self, path_and_query: impl AsRef<str>) -> hyper::Response<hyper::Body> {
        let request = self.get_request(path_and_query);
        self.client.request(request).await.unwrap()
    }

    pub async fn get_with(
        &self,
        path_and_query: impl AsRef<str>,
        f: impl Fn(&mut hyper::Request<hyper::Body>),
    ) -> hyper::Response<hyper::Body> {
        let mut request = self.get_request(path_and_query);
        f(&mut request);
        self.client.request(request).await.unwrap()
    }

    pub async fn post(
        &self,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::Body>,
    ) -> hyper::Response<hyper::Body> {
        let request = self.post_request(path_and_query, body);
        self.client.request(request).await.unwrap()
    }

    pub async fn patch(
        &self,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::Body>,
    ) -> hyper::Response<hyper::Body> {
        let request = self.patch_request(path_and_query, body);
        self.client.request(request).await.unwrap()
    }

    pub async fn patch_with(
        &self,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::Body>,
        f: impl Fn(&mut hyper::Request<hyper::Body>),
    ) -> hyper::Response<hyper::Body> {
        let mut request = self.patch_request(path_and_query, body);
        f(&mut request);
        self.client.request(request).await.unwrap()
    }

    pub async fn delete(&self, path_and_query: impl AsRef<str>) -> hyper::Response<hyper::Body> {
        let request = self.delete_request(path_and_query);
        self.client.request(request).await.unwrap()
    }

    pub async fn delete_with(
        &self,
        path_and_query: impl AsRef<str>,
        f: impl Fn(&mut hyper::Request<hyper::Body>),
    ) -> hyper::Response<hyper::Body> {
        let mut request = self.delete_request(path_and_query);
        f(&mut request);
        self.client.request(request).await.unwrap()
    }
}

impl Client {
    fn request(&self, path_and_query: impl AsRef<str>) -> hyper::http::request::Builder {
        let uri = hyper::Uri::builder()
            .scheme("http")
            .authority(self.uri.as_str())
            .path_and_query(path_and_query.as_ref())
            .build()
            .unwrap();

        hyper::Request::builder()
            .uri(uri)
            .header(utils::USER_HEADER, utils::USER)
    }

    fn get_request(&self, path_and_query: impl AsRef<str>) -> hyper::Request<hyper::Body> {
        self.request(path_and_query)
            .method(hyper::Method::GET)
            .body(hyper::Body::empty())
            .unwrap()
    }

    fn post_request(
        &self,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::Body>,
    ) -> hyper::Request<hyper::Body> {
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
        body: impl Into<hyper::Body>,
    ) -> hyper::Request<hyper::Body> {
        self.request(path_and_query)
            .method(hyper::Method::PATCH)
            .body(body.into())
            .unwrap()
    }

    fn delete_request(&self, path_and_query: impl AsRef<str>) -> hyper::Request<hyper::Body> {
        self.request(path_and_query)
            .method(hyper::Method::DELETE)
            .body(hyper::Body::empty())
            .unwrap()
    }
}
