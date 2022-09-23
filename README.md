# hatena-rs

Rust library for Hatena API.

## References

For the specifications of each API, refer to [Hatena Developer Center](https://developer.hatena.ne.jp/ja/documents/).

## Supported API

| status | API | note |
|--------|-----|------|
| ☀️ | [Hatena OAuth](https://developer.hatena.ne.jp/ja/documents/auth/) | OAuth v1.0a only, WSSE is not supported |
| ☁️ | [Hatena Fotolife](https://developer.hatena.ne.jp/ja/documents/fotolife/) | post image |
| ⛈️ | [Hatena Star](https://developer.hatena.ne.jp/ja/documents/star/) |  |
| ⛈️ | [Hatena Blog](https://developer.hatena.ne.jp/ja/documents/blog/) |  |
| ⛈️ | [Mackerel](https://developer.hatena.ne.jp/ja/documents/mackerel) |  |
| ⛈️ | [Hatena Bookmark](https://developer.hatena.ne.jp/ja/documents/bookmark/) |  |

## Usage

```rs
/// OAuth
use hatena_rs::oauth::{HatenaOauth, HatenaConsumerInfo consts::OauthScope};
let scopes = vec![
  OauthScope::WritePublic,
  OauthScope::WritePrivate,
  OauthScope::ReadPublic,
  OauthScope::ReadPrivate,
];
let consumer_info = HatenaConsumerInfo::from_env()?;
let mut oauth = HatenaOauth::new(scopes, None, consumer_info)?;
let access_token = oauth.get_access_token(true)?;

/// Fotolife
use hatena_rs::fotolife::Fotolife;
let fotolife = Fotolife::new(oauth);
fotolife.post_image("./kirby.png", "title", 30)?;
```
