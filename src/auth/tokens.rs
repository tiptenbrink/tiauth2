use crate::error::Error;

pub struct Tokens {

}

pub async fn new_token_family() -> Result<Tokens, Error> {
    Ok(Tokens {})
}