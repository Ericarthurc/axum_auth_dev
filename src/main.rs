use askama::Template;
use axum::{
    body::{self, Full},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Form, Router,
};
use http::StatusCode;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::Deserialize;
use std::net::SocketAddr;
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};

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
        .route("/", get(index).post(index_form))
        .route("/authenticated", get(auth))
        .layer(CookieManagerLayer::new());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Template)]
#[template(path = "login.html")]
struct IndexTemplate {}

async fn index() -> impl IntoResponse {
    let template = IndexTemplate {};

    HtmlTemplate(template)
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Input {
    username: String,
    password: String,
}

async fn index_form(Form(input): Form<Input>) -> impl IntoResponse {
    dbg!(&input);
    if input.password != "password1234" {
        Redirect::to("/").into_response()
    } else {
        Redirect::to("/authenticated").into_response()
    }
}

#[derive(Template)]
#[template(path = "auth.html")]
struct AuthTemplate {}

async fn auth() -> impl IntoResponse {
    let template = AuthTemplate {};

    HtmlTemplate(template)
}

// async fn cookie_adder(cookies: Cookies) -> impl IntoResponse {
//     cookies.add(
//         Cookie::build("id", "vb2384b5v38n4")
//             .secure(true)
//             .http_only(true)
//             .finish(),
//     );
//     (StatusCode::OK, Html("<h1>Index</h1>"))
// }
