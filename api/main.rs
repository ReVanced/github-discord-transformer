use hmac_sha256::HMAC;
use http::StatusCode;
use vercel_runtime::{Body, Request, Response, run, service_fn};

const HEADER_EVENT: &str = "X-GitHub-Event";
const HEADER_SIGNATURE: &str = "X-Hub-Signature-256";
const EVENT_PING: &[u8] = b"ping";
const ACTION_CREATED: &str = "created";

#[tokio::main]
async fn main() -> std::result::Result<(), vercel_runtime::Error> {
    run(service_fn(handle_webhook)).await
}

async fn handle_webhook(req: Request) -> Result<Response<Body>> {
    let body = read_body(&req).await?;

    verify_signature(&req, &body)?;

    if !is_ping_event(&req) {
        let event = parse_event(&body)?;
        if event.action == ACTION_CREATED {
            notify_new_sponsor(&event).await?;
        }
    }

    ok_response()
}

async fn read_body(req: &Request) -> Result<Vec<u8>> {
    let body = req.body().clone();
    let bytes = vercel_runtime::body::to_bytes(body)
        .await
        .map_err(|_| Error("body-read-error"))?;
    Ok(bytes.to_vec())
}

fn parse_event(body: &[u8]) -> Result<GithubEvent> {
    serde_json::from_slice(body).map_err(|_| Error("invalid-json"))
}

fn is_ping_event(req: &Request) -> bool {
    req.headers()
        .get(HEADER_EVENT)
        .is_some_and(|v| v.as_bytes() == EVENT_PING)
}

fn verify_signature(req: &Request, body: &[u8]) -> Result<()> {
    let expected = get_header(req, HEADER_SIGNATURE)?;
    let secret = get_env("GITHUB_SECRET")?;

    let computed = compute_signature(body, secret.as_bytes());

    if computed.as_bytes() != expected {
        return Err(Error("invalid-signature"));
    }

    Ok(())
}

fn compute_signature(body: &[u8], secret: &[u8]) -> String {
    let mac = HMAC::mac(body, secret);
    format!("sha256={}", hex::encode(mac))
}

async fn notify_new_sponsor(event: &GithubEvent) -> Result<()> {
    let sponsor = &event.sponsorship.sponsor;
    let amount = event.sponsorship.tier.monthly_price_in_dollars;

    notify_discord(&sponsor.login, &sponsor.html_url, amount).await
}

const COLOR_SUCCESS: u32 = 0x00FF00;

async fn notify_discord(username: &str, profile_url: &str, amount_usd: i64) -> Result<()> {
    DiscordWebhook::new(get_env("DISCORD_WEBHOOK_URL")?)
        .unwrap()
        .send(&Message::new(|message| {
            message
                .embed(|embed| {
                    embed
                        .title("New Sponsor!")
                        .description(format!(
                            "[{username}]({profile_url}) just donated ${amount_usd}!"
                        ))
                        .color(COLOR_SUCCESS)
                        .footer(|footer| footer.text("Sponsorship Notifications"))
                })
                .allowed_mentions(|am| am.empty())
        }))
        .await
}

fn ok_response() -> Result<Response<Body>> {
    Response::builder().status(StatusCode::OK).body(Body::Empty)
}

fn get_header<'a>(req: &'a Request, name: &str) -> Result<&'a [u8]> {
    req.headers()
        .get(name)
        .map(|v| v.as_bytes())
        .ok_or(Error("missing-header"))
}

fn get_env(name: &str) -> Result<String> {
    std::env::var(name).map_err(|_| Error("missing-env-var"))
}

#[derive(Debug, Deserialize)]
pub struct GithubEvent {
    pub action: String,
    pub sponsorship: Sponsorship,
}

#[derive(Debug, Deserialize)]
pub struct Sponsorship {
    pub sponsor: Sponsor,
    pub tier: Tier,
}

#[derive(Debug, Deserialize)]
pub struct Sponsor {
    pub login: String,
    pub html_url: String,
}

#[derive(Debug, Deserialize)]
pub struct Tier {
    pub monthly_price_in_dollars: i64,
}
