use super::error::Error;

pub mod time {
    use super::Error;

    // Allowed because u64 millis is already many times the age of the universe
    #[allow(clippy::cast_possible_truncation)]
    pub fn serialize(time: &std::time::SystemTime) -> String {
        let millis = time
            .duration_since(std::time::UNIX_EPOCH)
            // Allowed because this number is unsigned and can never go back in time
            .unwrap()
            .as_millis();

        let mut buffer = itoa::Buffer::new();
        buffer.format(millis as u64).to_owned()
    }

    pub fn deserialize(timestamp: &str) -> Result<std::time::SystemTime, Error> {
        let millis = timestamp.parse::<u64>()? + 1;
        Ok(std::time::UNIX_EPOCH
            .checked_add(std::time::Duration::from_millis(millis))
            // Allowed because a u64 millis duration may never overflow
            .unwrap())
    }
}

pub mod request {
    use super::Error;
    use crate::store;

    pub(in crate::server) const USER_HEADER: &str = "x-user";

    #[derive(
        serde::Deserialize, gotham_derive::StateData, gotham_derive::StaticResponseExtender,
    )]
    pub struct Id {
        pub id: store::Id,
    }

    pub struct User;

    impl User {
        pub fn borrow_from(state: &gotham::state::State) -> Result<&str, Error> {
            use gotham::state::FromState;

            gotham::hyper::HeaderMap::borrow_from(state)
                .get(USER_HEADER)
                .ok_or(Error::MissingUser)
                .and_then(|header| header.to_str().map_err(|_| Error::BadHeader))
        }
    }

    #[derive(
        gotham_derive::StateData, serde::Deserialize, gotham_derive::StaticResponseExtender,
    )]
    pub struct Limit {
        pub limit: Option<u32>,
    }

    pub struct UnmodifiedSince;

    impl UnmodifiedSince {
        pub fn borrow_from(state: &gotham::state::State) -> Result<std::time::SystemTime, Error> {
            use gotham::state::FromState;

            gotham::hyper::HeaderMap::borrow_from(state)
                .get(gotham::hyper::header::IF_UNMODIFIED_SINCE)
                .ok_or(Error::OutOfSync)
                .and_then(|header| header.to_str().map_err(|_| Error::BadHeader))
                .and_then(super::time::deserialize)
        }
    }

    pub struct Body;

    impl Body {
        pub async fn take_from<D: store::Data>(
            state: &mut gotham::state::State,
        ) -> Result<D, Error> {
            use gotham::hyper;
            use gotham::state::FromState;

            let request_length = hyper::HeaderMap::borrow_from(state)
                .get(hyper::header::CONTENT_LENGTH)
                .and_then(|len| len.to_str().ok())
                .and_then(|len| len.parse::<usize>().ok())
                .ok_or(Error::ContentLengthMissing)?;

            if request_length > 1024 {
                return Err(Error::PayloadTooLarge);
            }

            let body = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                hyper::body::to_bytes(hyper::Body::take_from(state)),
            )
            .await
            .map_err(|_| Error::ReadTimeout)?
            .map_err(Error::Hyper)?;
            serde_json::from_slice(&body).map_err(Error::JsonDeserialize)
        }
    }
}
