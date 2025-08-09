pub mod api_response;
pub mod cache;
pub mod form_extractor;
pub mod query_params;

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

extern crate bcrypt;

use bcrypt::{hash, verify, DEFAULT_COST};

// Helper functions
fn hash_password(password: &str) -> String {
    hash(password, DEFAULT_COST).unwrap()
}

fn verify_password(input: &str, stored: &str) -> bool {
    verify(input, stored).unwrap_or(false)
}

/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    company: String,
    exp: usize,
}

pub fn generate_token(sub: &str, company: &str) -> String {
    let claims = Claims {
        sub: sub.to_string(),
        company: company.to_string(),
        exp: 10000000000,
    };

    let header = Header::new(Algorithm::HS256);
    let encoding_key = EncodingKey::from_secret("secret".as_ref());
    encode(&header, &claims, &encoding_key).unwrap()
}
