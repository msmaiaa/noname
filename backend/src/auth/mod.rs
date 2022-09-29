use argon2::{self, Config};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header};
use std::env;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TokenData {
    pub id: i32,
    pub iat: i64,
    pub exp: i64,
}

pub fn create_access_token(id: i32) -> String {
    let iat = Utc::now();
    let exp = iat + Duration::seconds(3600);
    let iat = iat.timestamp_millis();
    let exp = exp.timestamp_millis();

    let key = EncodingKey::from_secret(env::var("JWT_KEY").expect("JWT_KEY not set").as_bytes());
    let claims = TokenData { id, iat, exp };
    let header = Header::new(Algorithm::HS256);
    encode(&header, &claims, &key).expect("Failed to create access token")
}

pub fn decode_token(token: &str) -> Option<TokenData> {
    let key = DecodingKey::from_secret(env::var("JWT_KEY").expect("JWT_KEY not set").as_bytes());
    let res = decode::<TokenData>(&token, &key, &jsonwebtoken::Validation::default());
    match res {
        Ok(data) => Some(data.claims),
        Err(_) => None,
    }
}

pub fn encrypt(password: &str) -> String {
    //	the salt must have atleast 16 characters
    let salt = env::var("SALT").unwrap_or("123451234512345123451235".to_string());
    let config = Config::default();
    argon2::hash_encoded(password.as_bytes(), salt.as_bytes(), &config)
        .expect("Failed to hash password")
}

pub fn compare_hash(password: &str, encrypted: &str) -> bool {
    encrypt(&password) == encrypted.to_string()
}
