mod consts;
mod error;
mod fotolife;

use std::io::Read;
use std::path::Path;

use crate::fotolife::consts::*;
use crate::fotolife::error::*;
use crate::fotolife::fotolife::*;
use crate::oauth::HatenaOauth;

use reqwest::StatusCode;
use scraper::{Html, Selector};

/// Hatena Fotolife client instance
pub struct Fotolife {
  // OAuth manager client
  pub oauth: HatenaOauth,
}

impl Fotolife {
  /// Create a new Fotolife client instance
  ///
  /// # Arguments
  ///
  /// * `access_token` - Access token for Hatena API
  pub fn new(oauth: HatenaOauth) -> Self {
    Self { oauth }
  }

  /// Upload a photo to Hatena Fotolife
  ///
  /// # Arguments
  ///
  /// * `image_path`: Path to the image file to upload
  /// * `title` - Title of the photo
  /// * `timeout` - Timeout in seconds
  pub fn post_image(
    &mut self,
    image_path: &Path,
    title: &str,
    timeout: u64,
  ) -> Result<FotolifePostResponse, FotolifeError> {
    let xml = self.generate_post_xml(image_path, title, "hatena-rs")?;
    let res = self.oauth.post(FOTOLIFE_URL_POST, &xml, false, timeout)?;

    if res.status().is_success() {
      let location = res
        .headers()
        .get("location")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
      Ok(FotolifePostResponse::new(
        location.split("/").last().unwrap().to_string(),
      ))
    } else {
      return Err(FotolifeError::UploadFailure {
        status: res.status(),
      });
    }
  }

  /// Get image from Fotolife
  ///
  /// TODO: unimplemented
  ///
  /// # Arguments
  ///
  /// * `image_id` - ID of the image to get
  #[allow(dead_code)]
  pub fn get_image(&mut self, photo_id: &str) -> Result<(), FotolifeError> {
    let url = format!("{}/{}", FOTOLIFE_URL_EDIT, photo_id);
    let res = self.oauth.get(&url, false)?; // XXX

    unimplemented!();

    Ok(())
  }

  /// List image in specific `path` of user's Fotolife using Cookie.
  ///
  /// When success, returns a list of image IDs.
  ///
  /// Note that this doesn't use API and needs Cookie (`rk`).
  ///
  /// # Arguments
  ///
  /// * `path` - Path to list images
  /// * `cookie` - logged-in Cookie named `rk`
  /// * `username` - Hatena username. If not specified, it uses OAuth API to fetch username.
  ///
  ///
  pub fn list_images_directory(
    &mut self,
    path: &str,
    cookie: &str,
    username: Option<&str>,
  ) -> Result<Vec<String>, FotolifeError> {
    let username = if username.is_none() {
      let me_info = self.oauth.get_access_token(false)?;
      me_info.url_name
    } else {
      username.unwrap().into()
    };

    let url = format!("{}/{}/{}/", FOTOLIFE_URL_LIST, username, path);
    let res = reqwest::blocking::Client::new()
      .get(&url)
      .header("Cookie", format!("rk={}", cookie))
      .send()?;

    match res.status() {
      StatusCode::OK => {
        let body = res.text().unwrap();
        let image_ids = self.parse_photolist_html(&body, &username);
        Ok(image_ids)
      }
      StatusCode::NOT_FOUND => Ok(vec![]),
      _ => Err(FotolifeError::UploadFailure {
        status: res.status(),
      }),
    }
  }

  fn generate_post_xml(
    &self,
    image_path: &Path,
    title: &str,
    generator: &str,
  ) -> Result<String, FotolifeError> {
    if !image_path.exists() || !image_path.is_file() {
      return Err(FotolifeError::ResourceNotFound {
        resource: image_path.to_string_lossy().to_string(),
      });
    }

    let file = std::fs::File::open(image_path).map_err(|_| FotolifeError::ResourceNotFound {
      resource: image_path.to_string_lossy().to_string(),
    })?;
    let bytes = file.bytes().map(|b| b.unwrap()).collect::<Vec<u8>>();
    let encoded_image = base64::encode(&bytes);
    let typestr = format!(
      "image/{}",
      image_path.extension().unwrap().to_str().unwrap()
    );

    Ok(format!(
      r#"
        <entry xmlns="http://purl.org/atom/ns#">
          <title>{}</title>
          <content mode="base64" type="{}">{}</content>
          <generator>{}</generator>
        </entry>
      "#,
      title, typestr, encoded_image, generator,
    ))
  }

  fn parse_photolist_html(&self, html: &str, user_name: &str) -> Vec<String> {
    let mut photos = vec![];
    let document = Html::parse_document(html);
    let selector = Selector::parse("img.foto_thumb").unwrap();

    for element in document.select(&selector) {
      let a_elem = if let Some(a_elem) = element.parent() {
        a_elem.value().as_element().unwrap()
      } else {
        continue;
      };
      let href = if let Some(href) = a_elem.attr("href") {
        href
      } else {
        continue;
      };
      if href.starts_with(&format!("/{}/", user_name)) {
        let id = href.split("/").last().unwrap();
        photos.push(id.into());
      }
    }

    photos
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::oauth::consts::OauthScope;
  use crate::oauth::{HatenaConsumerInfo, HatenaOauth};

  #[test]
  fn test_post_image() {
    let consumer_info = HatenaConsumerInfo::from_env().unwrap();
    let oauth = HatenaOauth::new(
      vec![
        OauthScope::WritePublic,
        OauthScope::WritePrivate,
        OauthScope::ReadPublic,
        OauthScope::ReadPrivate,
      ],
      None,
      consumer_info,
    )
    .unwrap();
    let mut fotolife = Fotolife::new(oauth);

    let res = fotolife
      .post_image(Path::new("test.png"), "test rust", 10)
      .unwrap();
    println!("{:?}", res);
  }

  #[test]
  fn test_get_image() {
    let consumer_info = HatenaConsumerInfo::from_env().unwrap();
    let oauth = HatenaOauth::new(
      vec![
        OauthScope::WritePublic,
        OauthScope::WritePrivate,
        OauthScope::ReadPublic,
        OauthScope::ReadPrivate,
      ],
      None,
      consumer_info,
    )
    .unwrap();
    let mut fotolife = Fotolife::new(oauth);

    fotolife.get_image("hogehoge").unwrap();
  }

  #[test]
  fn test_list_photos() {
    let consumer_info = HatenaConsumerInfo::from_env().unwrap();
    let oauth = HatenaOauth::new(
      vec![
        OauthScope::WritePublic,
        OauthScope::WritePrivate,
        OauthScope::ReadPublic,
        OauthScope::ReadPrivate,
      ],
      None,
      consumer_info,
    )
    .unwrap();
    let mut fotolife = Fotolife::new(oauth);
    let cookie = std::env::var("FOTOLIFE_COOKIE").unwrap();

    let ids = fotolife
      .list_images_directory("hatena-rs", &cookie, Some("smallkirby"))
      .unwrap();
    println!("{:?}", ids);
  }
}
