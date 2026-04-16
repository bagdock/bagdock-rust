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

## Configuration

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `api_key` | `&str` | — | **Required.** Your Bagdock API key |
| `with_base_url(url)` | `&str` | `https://api.bagdock.com/api/v1` | API base URL |

## License

MIT — see [LICENSE](LICENSE)
