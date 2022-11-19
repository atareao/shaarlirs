use std::collections::HashSet;
use actix_web::http::header::HeaderMap;
use serde::{Serialize, Deserialize};
use thiserror::Error;

use log::debug;
use jsonwebtoken::{decode, TokenData};

const BEARER: &str = "Bearer ";
const AUTHORIZATION: &str = "authorization";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    iat: i64,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("wrong credentials")]
    WrongCredentialsError,
    #[error("jwt token not valid")]
    JWTTokenError,
    #[error("no auth header")]
    NoAuthHeaderError,
    #[error("invalid auth header")]
    InvalidAuthHeaderError,
}

#[derive(Serialize, Debug)]
struct ErrorResponse {
    message: String,
    status: String,
}

fn jwt_from_header(headers: &HeaderMap) -> Result<String, Error> {
    let header = match headers.get(AUTHORIZATION) {
        Some(v) => v,
        None => return Err(Error::NoAuthHeaderError),
    };
    let auth_header = match std::str::from_utf8(header.as_bytes()) {
        Ok(v) => v,
        Err(_) => return Err(Error::NoAuthHeaderError),
    };
    if !auth_header.starts_with(BEARER) {
        return Err(Error::InvalidAuthHeaderError);
    }
    Ok(auth_header.trim_start_matches(BEARER).to_owned())
}

pub fn authorize(headers: &HeaderMap, secret: &str) -> Result<TokenData<Claims>, Error>{
    match jwt_from_header(headers){
        Ok(jwt) => {
            let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS512);
            validation.required_spec_claims = HashSet::new();
            debug!("Secret: {}", secret);
            let key = jsonwebtoken::DecodingKey::from_secret(secret.as_ref());
            match decode::<Claims>(&jwt, &key, &validation) {
                Ok(v) => Ok(v),
                Err(_) => Err(Error::JWTTokenError),
            }
        },
        Err(e) => Err(e)
    }
}
