use crate::{AppState, OidcFlow, WebSession};
use axum::{
    extract::{Query, Request, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Redirect, Response},
};
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
    reqwest, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointMaybeSet,
    EndpointNotSet, EndpointSet, IssuerUrl, Nonce, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, Scope, TokenResponse,
};
use serde::Deserialize;
use serde_json::json;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use uuid::Uuid;

const FLOW_TTL: Duration = Duration::from_secs(600);
const SESSION_TTL: Duration = Duration::from_secs(86_400);
const SECURE_SESSION_COOKIE: &str = "__Host-hash-session-v2";
const LOCAL_SESSION_COOKIE: &str = "hash-session-v2";
const LEGACY_SESSION_COOKIE: &str = "hash-session";

fn oidc_values(state: &AppState) -> Option<(&str, &str, &str, &str)> {
    Some((
        state.config.auth.oidc_issuer.as_deref()?,
        state.config.auth.oidc_client_id.as_deref()?,
        state.config.auth.oidc_client_secret.as_deref()?,
        state.config.auth.oidc_redirect_url.as_deref()?,
    ))
}

type ConfiguredClient = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

async fn oidc_client(state: &AppState) -> Result<(ConfiguredClient, reqwest::Client), StatusCode> {
    let (issuer, client_id, secret, redirect) = oidc_values(state).ok_or(StatusCode::NOT_FOUND)?;
    let http = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let metadata = CoreProviderMetadata::discover_async(
        IssuerUrl::new(issuer.to_string()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        &http,
    )
    .await
    .map_err(|error| {
        tracing::error!("OIDC discovery failed: {error}");
        StatusCode::BAD_GATEWAY
    })?;
    let client = CoreClient::from_provider_metadata(
        metadata,
        ClientId::new(client_id.to_string()),
        Some(ClientSecret::new(secret.to_string())),
    )
    .set_redirect_uri(
        RedirectUrl::new(redirect.to_string()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );
    Ok((client, http))
}

pub async fn oidc_login(State(state): State<Arc<AppState>>) -> Result<Redirect, StatusCode> {
    let (client, _) = oidc_client(&state).await?;
    let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
    let mut request = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .set_pkce_challenge(challenge);
    for scope in state.config.auth.oidc_scopes.split_whitespace() {
        request = request.add_scope(Scope::new(scope.to_string()));
    }
    let (url, csrf, nonce) = request.url();
    let now = Instant::now();
    let mut flows = state
        .oidc_flows
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    flows.retain(|_, flow| now.duration_since(flow.created) < FLOW_TTL);
    flows.insert(
        csrf.secret().clone(),
        OidcFlow {
            nonce: nonce.secret().clone(),
            pkce_verifier: verifier.secret().clone(),
            created: now,
        },
    );
    Ok(Redirect::temporary(url.as_str()))
}

#[derive(Deserialize)]
pub struct OidcCallback {
    code: String,
    state: String,
}

pub async fn oidc_callback(
    State(state): State<Arc<AppState>>,
    Query(query): Query<OidcCallback>,
) -> Result<Response, StatusCode> {
    let flow = state
        .oidc_flows
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .remove(&query.state)
        .filter(|flow| flow.created.elapsed() < FLOW_TTL)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let (client, http) = oidc_client(&state).await?;
    let tokens = client
        .exchange_code(AuthorizationCode::new(query.code))
        .map_err(|_| StatusCode::BAD_GATEWAY)?
        .set_pkce_verifier(PkceCodeVerifier::new(flow.pkce_verifier))
        .request_async(&http)
        .await
        .map_err(|error| {
            tracing::warn!("OIDC code exchange failed: {error}");
            StatusCode::BAD_GATEWAY
        })?;
    let id_token = tokens.id_token().ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = id_token
        .claims(&client.id_token_verifier(), &Nonce::new(flow.nonce))
        .map_err(|error| {
            tracing::warn!("OIDC ID token validation failed: {error}");
            StatusCode::UNAUTHORIZED
        })?;
    let session_id = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    state
        .web_sessions
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .insert(
            session_id.clone(),
            WebSession {
                subject: claims.subject().as_str().to_string(),
                created: Instant::now(),
            },
        );
    let secure = uses_secure_cookie(&state);
    let cookie = session_cookie(&state, &session_id);
    let mut response = Redirect::to("/").into_response();
    append_set_cookie(&mut response, &cookie)?;
    append_set_cookie(
        &mut response,
        &expired_cookie(LEGACY_SESSION_COOKIE, secure),
    )?;
    Ok(response)
}

pub async fn auth_status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let authenticated = valid_session(&state, &headers);
    let body =
        json!({ "oidc_enabled": oidc_values(&state).is_some(), "authenticated": authenticated });
    let mut response = Json(body).into_response();
    append_set_cookie(
        &mut response,
        &expired_cookie(LEGACY_SESSION_COOKIE, uses_secure_cookie(&state)),
    )?;
    Ok(response)
}

pub async fn logout(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    if let Ok(mut sessions) = state.web_sessions.lock() {
        for name in [session_cookie_name(&state), LEGACY_SESSION_COOKIE] {
            if let Some(id) = cookie_value(&headers, name) {
                sessions.remove(id);
            }
        }
    }
    let mut response = StatusCode::NO_CONTENT.into_response();
    let secure = uses_secure_cookie(&state);
    for name in [session_cookie_name(&state), LEGACY_SESSION_COOKIE] {
        if let Ok(value) = HeaderValue::from_str(&expired_cookie(name, secure)) {
            response.headers_mut().append(header::SET_COOKIE, value);
        }
    }
    response
}

pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let bearer_ok = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .is_some_and(|key| key == state.config.auth.api_key);
    if bearer_ok || valid_session(&state, request.headers()) {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

/// MCP and non-browser clients deliberately remain API-key only.
pub async fn require_api_key(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let valid = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .is_some_and(|key| key == state.config.auth.api_key);
    if valid {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

fn valid_session(state: &AppState, headers: &HeaderMap) -> bool {
    let Some(id) = cookie_value(headers, session_cookie_name(state)) else {
        return false;
    };
    let Ok(mut sessions) = state.web_sessions.lock() else {
        return false;
    };
    sessions.retain(|_, session| session.created.elapsed() < SESSION_TTL);
    sessions.contains_key(id)
}

fn uses_secure_cookie(state: &AppState) -> bool {
    state
        .config
        .auth
        .oidc_redirect_url
        .as_deref()
        .is_some_and(|url| url.starts_with("https://"))
}

fn session_cookie_name(state: &AppState) -> &'static str {
    if uses_secure_cookie(state) {
        SECURE_SESSION_COOKIE
    } else {
        LOCAL_SESSION_COOKIE
    }
}

fn session_cookie(state: &AppState, session_id: &str) -> String {
    let secure = uses_secure_cookie(state);
    format!(
        "{}={session_id}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}{}",
        session_cookie_name(state),
        SESSION_TTL.as_secs(),
        if secure { "; Secure" } else { "" }
    )
}

fn expired_cookie(name: &str, secure: bool) -> String {
    format!(
        "{name}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT{}",
        if secure { "; Secure" } else { "" }
    )
}

fn append_set_cookie(response: &mut Response, cookie: &str) -> Result<(), StatusCode> {
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(cookie).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );
    Ok(())
}

fn cookie_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|part| {
            let (key, value) = part.trim().split_once('=')?;
            (key == name).then_some(value)
        })
}
