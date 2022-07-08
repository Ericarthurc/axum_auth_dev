use askama::Template;
use axum::{
    body::{self, Full},
    extract,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, get_service, post, Route},
    Form, Router,
};
use http::StatusCode;
use jsonwebtoken::{
    decode, encode, errors::ErrorKind, Algorithm, DecodingKey, EncodingKey, Header, TokenData,
    Validation,
};
use serde::{Deserialize, Serialize};
use std::{io::Error, net::SocketAddr};
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

const JWT_SECRET: &str = "fb23985y982fh75987jj23fbvngijeorcjgih";

pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(body::boxed(Full::from(format!(
                    "Failed to render template. Error: {}",
                    err
                ))))
                .unwrap(),
        }
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/login", post(index_form))
        .route("/authenticated", get(auth))
        .layer(CookieManagerLayer::new())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .nest(
            "/public",
            get_service(ServeDir::new("./public/")).handle_error(
                |error: std::io::Error| async move {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled internal error: {}", error),
                    )
                },
            ),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Template)]
#[template(path = "login.html")]
struct IndexTemplate {}

async fn index(cookies: Cookies) -> Response {
    let template = IndexTemplate {};

    // let id = cookies.get("id").unwrap().value().to_string();
    let id = match cookies.get("id") {
        Some(id) => id.value().to_string(),
        None => return HtmlTemplate(template).into_response(),
    };

    let claims = match validate_jwt(id) {
        Ok(c) => c,
        Err(_) => return HtmlTemplate(template).into_response(),
    };

    if claims.claims.id == "password1234" {
        return Redirect::to("/authenticated").into_response();
    }

    HtmlTemplate(template).into_response()
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Input {
    username: String,
    password: String,
}

async fn index_form(extract::Json(payload): extract::Json<Input>, cookies: Cookies) -> Response {
    println!("{:?}", payload);

    // TEMP CODE, WILL GRAB AUTH FROM DATABASE
    if payload.password != "password1234" {
        return (StatusCode::FORBIDDEN).into_response();
    } else {
        let token = generate_jwt(payload.password).unwrap();

        cookies.add(
            Cookie::build("id", token)
                .secure(true)
                .http_only(true)
                .finish(),
        );

        return (StatusCode::ACCEPTED).into_response();
    }
}

#[derive(Template)]
#[template(path = "auth.html")]
struct AuthTemplate {}

async fn auth(cookies: Cookies) -> Response {
    let template = AuthTemplate {};

    let id = match cookies.get("id") {
        Some(id) => id.value().to_string(),
        None => return Redirect::to("/").into_response(),
    };

    match validate_jwt(id) {
        Ok(c) => match c.claims.id.as_str() {
            "password1234" => HtmlTemplate(template).into_response(),
            _ => Redirect::to("/").into_response(),
        },
        Err(_) => Redirect::to("/").into_response(),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    id: String,
    exp: usize,
}

fn generate_jwt(user_id: String) -> Result<String, jsonwebtoken::errors::Error> {
    match encode(
        &Header::default(),
        &Claims {
            id: user_id,
            exp: 100000000000000,
        },
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    ) {
        Ok(t) => Ok(t),
        Err(err) => Err(err),
    }
}

fn validate_jwt(token: String) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    let validation = Validation::new(Algorithm::HS256);

    match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &validation,
    ) {
        Ok(c) => Ok(c),
        Err(err) => match *err.kind() {
            ErrorKind::InvalidToken => Err(err),
            ErrorKind::ExpiredSignature => Err(err),
            _ => Err(err),
        },
    }
}
