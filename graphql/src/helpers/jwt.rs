use std::sync::Arc;

use chrono::{Duration, Utc};
use jsonwebtoken::{
    errors::Error, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
}

#[derive(Clone)]
pub struct JWT(Arc<JWTInner>);

#[derive(Clone)]
struct JWTInner {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    algo: Algorithm,
}

impl JWT {
    pub fn new_from_pem(private: &[u8], public: &[u8]) -> Result<Self, Error> {
        Ok(JWT(Arc::new(JWTInner {
            algo: Algorithm::ES256,
            encoding_key: EncodingKey::from_ec_pem(private)?,
            decoding_key: DecodingKey::from_ec_pem(public)?,
        })))
    }

    pub fn encode(&self, address: String, ttl: Option<i64>) -> Result<String, Error> {
        let exp = Utc::now() + Duration::days(ttl.unwrap_or(90));
        let claim = Claims {
            sub: address,
            exp: exp.timestamp(),
        };

        self.encode_with_claims(&claim)
    }

    pub fn encode_with_claims(&self, claims: &Claims) -> Result<String, Error> {
        jsonwebtoken::encode(&Header::new(self.0.algo), claims, &self.0.encoding_key)
    }

    pub fn decode(&self, token: String) -> Result<TokenData<Claims>, Error> {
        jsonwebtoken::decode::<Claims>(
            token.as_ref(),
            &self.0.decoding_key,
            &Validation::new(self.0.algo),
        )
    }
}
