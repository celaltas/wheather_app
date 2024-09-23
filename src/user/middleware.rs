use poem::{
    http::StatusCode,
    web::headers::{self, authorization::Bearer, HeaderMapExt},
    Endpoint, Error, Middleware, Request, Response, Result,
};

use crate::{configuration::JwtSettings, token::verify_token};

pub struct JWTAuth {
    conf: JwtSettings,
}

impl JWTAuth {
    pub fn new(conf: JwtSettings) -> Self {
        JWTAuth { conf }
    }
}

impl<E: Endpoint> Middleware<E> for JWTAuth {
    type Output = JwtAuthEndpoint<E>;

    fn transform(&self, ep: E) -> Self::Output {
        JwtAuthEndpoint {
            ep,
            conf: self.conf.clone(),
        }
    }
}

pub struct JwtAuthEndpoint<E> {
    ep: E,
    conf: JwtSettings,
}

impl<E: Endpoint> Endpoint for JwtAuthEndpoint<E> {
    type Output = E::Output;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        let url_path = req.uri().path().to_string();

        if url_path == "/api/weather" && req.method() == "GET" {
            if let Some(auth) = req.headers().typed_get::<headers::Authorization<Bearer>>() {
                let res = verify_token(auth.token(), &self.conf);
                if res.is_ok() {
                    return self.ep.call(req).await;
                }
            } else {
                let error_message = "Authentication failed. Please provide a valid JWT token.";
                let response = Err(Error::from_response(
                    Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .content_type("text/plain")
                        .body(error_message.to_string()),
                ));
                return response;
            }
        }

        self.ep.call(req).await
    }
}
