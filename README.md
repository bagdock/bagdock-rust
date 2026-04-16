```
  ----++                                ----++                    ---+++     
  ---+++                                ---++                     ---++      
 ----+---     -----     ---------  --------++ ------     -----   ----++----- 
 ---------+ --------++----------++--------+++--------+ --------++---++---++++
 ---+++---++ ++++---++---+++---++---+++---++---+++---++---++---++------++++  
----++ ---++--------++---++----++---++ ---++---++ ---+---++     -------++    
----+----+---+++---++---++----++---++----++---++---+++--++ --------+---++   
---------++--------+++--------+++--------++ -------+++ -------++---++----++  
 +++++++++   +++++++++- +++---++   ++++++++    ++++++    ++++++  ++++  ++++  
                     --------+++                                             
                       +++++++                                               
```

# bagdock-rust

The official Rust SDK for the Bagdock API — manage facilities, contacts, tenancies, invoices, marketplace listings, and loyalty programs with async/await and strong typing.

[![crates.io](https://img.shields.io/crates/v/bagdock.svg)](https://crates.io/crates/bagdock)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Install

Add to your `Cargo.toml`:

```toml
[dependencies]
bagdock = "0.1"
```

## Quick start

```rust
use bagdock::Bagdock;

#[tokio::main]
async fn main() -> bagdock::Result<()> {
    let client = Bagdock::new("sk_live_...")?;

    let facilities = client.operator.list_facilities().await?;
    println!("Found {} facilities", facilities.data.len());

    let contact = client.operator.create_contact(&serde_json::json!({
        "email": "jane@example.com",
        "first_name": "Jane",
        "last_name": "Doe"
    })).await?;

    Ok(())
}
```

## API reference

### `client.operator`

| Method | Description |
|--------|-------------|
| `list_facilities()` | List facilities |
| `get_facility(id)` | Get a facility |
| `list_contacts()` | List contacts |
| `create_contact(data)` | Create a contact |
| `list_units()` | List units |
| `list_invoices()` | List invoices |

### `client.marketplace`

| Method | Description |
|--------|-------------|
| `search()` | Search marketplace locations |
| `get_listing(id)` | Get a listing |
| `create_rental(data)` | Create a rental |

### `client.loyalty`

| Method | Description |
|--------|-------------|
| `list_members()` | List loyalty members |
| `award_points(data)` | Award points |

## Error handling

```rust
use bagdock::BagdockError;

match client.operator.get_facility("fac_nonexistent").await {
    Ok(facility) => println!("{:?}", facility),
    Err(BagdockError::Api { status, code, message }) => {
        eprintln!("API error {status}: {code} — {message}");
    }
    Err(e) => eprintln!("Error: {e}"),
}
```

## Authentication

The SDK supports three authentication modes: API keys, OAuth access tokens, and OAuth2 client credentials.

### API key

```rust
let client = bagdock::Bagdock::new("sk_live_...")?;
```

### OAuth access token

```rust
let client = bagdock::Bagdock::with_access_token("eyJhbGciOiJSUzI1NiIs...")?;
```

### Client credentials

```rust
let client = bagdock::Bagdock::with_client_credentials(
    "oac_your_client_id",
    "bdok_secret_your_secret",
    vec!["facilities:read".into(), "contacts:read".into()],
    None,
)?;
```

### OAuth2 helpers

Use the OAuth2 helper functions for authorization code (PKCE), token exchange, and device flow. For external integrations, Bagdock **connect webviews** open OAuth for access control, security, and insurance without you reimplementing the full redirect flow—use these helpers when your app drives the OAuth dance directly (for example, server-side or native clients).

```rust
use bagdock::{generate_pkce, build_authorize_url, exchange_code, device_authorize, poll_device_token};

let pkce = generate_pkce();
let url = build_authorize_url("oac_...", "https://your-app.com/callback", &pkce.code_challenge, Some("openid contacts:read"), None, None);

let tokens = exchange_code("oac_...", code, redirect_uri, &pkce.code_verifier, None, None).await?;

let device = device_authorize("bagdock-cli", Some("developer:read"), None).await?;
println!("Open {} and enter: {}", device.verification_uri, device.user_code);
let device_tokens = poll_device_token("bagdock-cli", &device.device_code, 5, 600, None).await?;
```

## Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `Bagdock::new(api_key)` | `&str` | — | Build client with a Bagdock API key |
| `Bagdock::with_access_token(token)` | `&str` | — | Build client with an OAuth access token |
| `Bagdock::with_client_credentials(client_id, client_secret, scopes, base_url?)` | — | — | Build client using OAuth2 client credentials (`scopes`: `Vec<String>`; `base_url`: optional override) |
| `with_base_url(url)` | `&str` | `https://api.bagdock.com/api/v1` | API base URL |

Use `Bagdock::new`, `Bagdock::with_access_token`, or `Bagdock::with_client_credentials`—only one authentication path is needed per client.

## Documentation

- [Full documentation](https://bagdock.com/docs)
- [Rust SDK quickstart](https://bagdock.com/docs/sdks/rust)
- [API reference](https://bagdock.com/docs/api)
- [OAuth2 / OIDC guide](https://bagdock.com/docs/auth/oauth2)

## License

MIT — see [LICENSE](LICENSE)
