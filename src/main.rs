use axum::{
    Form, Router, debug_handler,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use axum_macros::FromRef;
use axum_session::{Session, SessionConfig, SessionLayer, SessionNullPool, SessionStore};
use axum_template::{RenderHtml, engine::Engine};
use handlebars::Handlebars;
use rustlio::TwilioRestClient;
use rustlio::verify::{VerificationCheckRequestParams, Verify};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env};
use tower::ServiceBuilder;
use tower_http::services::{ServeDir, ServeFile};

type AppEngine = Engine<Handlebars<'static>>;

// Define your application shared state
#[derive(Clone, FromRef)]
struct AppState {
    config: HashMap<String, String>,
    engine: AppEngine,
}

#[derive(Deserialize, Serialize)]
struct VerificationStatus<'a> {
    status: bool,
    message: &'a str,
}

#[tokio::main]
async fn main() {
    // Load the environment variables
    dotenvy::dotenv().ok();

    let mut hbs = Handlebars::new();
    hbs.register_template_file("sign-in", "templates/forms/sign-in.hbs")
        .unwrap();
    hbs.register_template_file("verify", "templates/forms/verify-otp.hbs")
        .unwrap();
    hbs.register_template_file(
        "verification-status",
        "templates/forms/verification-status.hbs",
    )
    .unwrap();

    let app_config = HashMap::from([
        (
            String::from("TWILIO_ACCOUNT_SID"),
            env::var("TWILIO_ACCOUNT_SID").unwrap_or("Twilio Account SID was not set".to_string()),
        ),
        (
            String::from("TWILIO_AUTH_TOKEN"),
            env::var("TWILIO_AUTH_TOKEN").unwrap_or("Twilio Auth Token was not set".to_string()),
        ),
        (
            String::from("TWILIO_VERIFY_SERVICE_SID"),
            env::var("TWILIO_VERIFY_SERVICE_SID")
                .unwrap_or("Twilio Verify Service SID was not set".to_string()),
        ),
    ]);

    let config = SessionConfig::default();
    let session_store = SessionStore::<SessionNullPool>::new(None, config)
        .await
        .unwrap();

    let app = Router::new()
        .route("/", get(show_signin_form))
        .route("/", post(send_otp_code))
        .route("/verify", get(show_verify_otp_form))
        .route("/verify", post(verify_otp_code))
        .with_state(AppState {
            config: app_config,
            engine: Engine::from(hbs),
        })
        .nest_service(
            "/assets",
            ServiceBuilder::new().service(ServeDir::new("assets")),
        )
        .nest_service(
            "/favicon.ico",
            ServiceBuilder::new().service(ServeFile::new("assets/favicon.ico")),
        )
        .layer(SessionLayer::new(session_store));

    println!("Server starting on :8080");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[derive(Serialize)]
struct NoData {}

#[derive(Deserialize, Serialize)]
struct SignIn {
    phone: String,
}

#[derive(Deserialize, Serialize)]
struct VerifyOtp {
    code: String,
}

#[derive(Serialize)]
struct StringCaseData {
    text: String,
}

/// Renders the sign-in form where users can enter and submit their phone numbers to
/// request an OTP code
async fn show_signin_form(engine: AppEngine) -> impl IntoResponse {
    let text = stringcase::pascal_case("sna_client_token");
    RenderHtml("sign-in", engine, StringCaseData { text })
}

/// Sends an OTP code to the nominated phone number
///
#[debug_handler]
async fn send_otp_code(
    session: Session<SessionNullPool>,
    State(state): State<AppState>,
    Form(sign_in): Form<SignIn>,
) -> Redirect {
    if sign_in.phone.is_empty() {
        return Redirect::to("/");
    }
    session.set("phone", &sign_in.phone);

    let Some(account_sid) = state.config.get("TWILIO_ACCOUNT_SID") else {
        return Redirect::to("/");
    };
    let Some(auth_token) = state.config.get("TWILIO_AUTH_TOKEN") else {
        return Redirect::to("/");
    };

    let verify = Verify {
        client: &TwilioRestClient {
            account_sid,
            auth_token,
        },
        ..Default::default()
    };

    let Some(verify_sid) = state.config.get("TWILIO_VERIFY_SERVICE_SID") else {
        return Redirect::to("/");
    };

    let response = verify
        .send_verification_token(verify_sid, sign_in.phone.as_str(), "sms")
        .await;

    match response {
        Ok(token_response) => println!(
            "Successfully sent the token (status: {})",
            token_response.status.unwrap_or("unknown".to_string())
        ),
        Err(err) => println!("Unable to send token because: {}", err),
    }

    Redirect::to("/verify")
}

/// This renders the form where users can verify the OTP code that they have received via SMS
async fn show_verify_otp_form(engine: AppEngine) -> impl IntoResponse {
    RenderHtml("verify", engine, NoData {})
}

async fn verify_otp_code(
    engine: AppEngine,
    session: Session<SessionNullPool>,
    State(state): State<AppState>,
    Form(verify_otp): Form<VerifyOtp>,
) -> Response {
    let phone: std::option::Option<String> = session.get("phone");
    let Some(phone_number) = phone else {
        return Redirect::to("/").into_response();
    };

    if verify_otp.code.is_empty() {
        return Redirect::to("/verify").into_response();
    }

    let Some(account_sid) = state.config.get("TWILIO_ACCOUNT_SID") else {
        return Redirect::to("/verify").into_response();
    };
    let Some(auth_token) = state.config.get("TWILIO_AUTH_TOKEN") else {
        return Redirect::to("/verify").into_response();
    };
    let verify = Verify {
        client: &TwilioRestClient {
            account_sid,
            auth_token,
        },
        ..Default::default()
    };

    let check_params = VerificationCheckRequestParams {
        code: verify_otp.code,
        to: phone_number,
        verification_sid: "".to_string(),
        amount: "".to_string(),
        payee: "".to_string(),
        sna_client_token: "".to_string(),
    };

    let Some(verify_sid) = state.config.get("TWILIO_VERIFY_SERVICE_SID") else {
        return Redirect::to("/verify").into_response();
    };
    let response = verify
        .check_verification_token(verify_sid, check_params)
        .await;
    let verification_check_response = match response {
        Ok(response) => {
            println!("Successfully validated the token");
            println!("Here is the response: {:?}", response);
            response
        }
        Err(error) => {
            println!("Here is the error: {:?}", error);
            return Redirect::to("/verify").into_response();
        }
    };

    let check_status = verification_check_response
        .status
        .unwrap_or(String::from("unknown"));

    let message = if check_status == "approved" {
        "Verification was successful"
    } else {
        "Verification failed"
    };
    let status = check_status == "approved";

    let data = VerificationStatus { status, message };

    session.remove("phone");

    RenderHtml("verification-status", engine, data).into_response()
}
