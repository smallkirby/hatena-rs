mod consts;
mod error;
mod fotolife;

use std::io::Read;
use std::path::Path;

use crate::fotolife::consts::*;
use crate::fotolife::error::*;
use crate::fotolife::fotolife::*;
use crate::oauth::HatenaOauth;

/// Hatena Fotolife client instance
pub struct Fotolife {
  // OAuth manager client
  oauth: HatenaOauth,
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
  pub fn post_image(
    &mut self,
    image_path: &Path,
    title: &str,
  ) -> Result<FotolifePostResponse, FotolifeError> {
    let xml = self.generate_post_xml(image_path, title, "hatena-rs")?;
    let res = self.oauth.post(FOTOLIFE_URL_POST, &xml, false)?;

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
  pub fn get_image(&mut self, photo_id: &str) -> Result<(), FotolifeError> {
    let url = format!("{}/{}", FOTOLIFE_URL_EDIT, photo_id);
    let res = self.oauth.get(&url, false)?; // XXX

    unimplemented!();

    Ok(())
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
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::oauth::consts::OauthScope;
  use crate::oauth::HatenaOauth;

  #[test]
  fn test_post_image() {
    let oauth = HatenaOauth::new(
      vec![
        OauthScope::WritePublic,
        OauthScope::WritePrivate,
        OauthScope::ReadPublic,
        OauthScope::ReadPrivate,
      ],
      None,
    )
    .unwrap();
    let mut fotolife = Fotolife::new(oauth);

    let res = fotolife
      .post_image(Path::new("test.png"), "test rust")
      .unwrap();
    println!("{:?}", res);
  }

  #[test]
  fn test_get_image() {
    let oauth = HatenaOauth::new(
      vec![
        OauthScope::WritePublic,
        OauthScope::WritePrivate,
        OauthScope::ReadPublic,
        OauthScope::ReadPrivate,
      ],
      None,
    )
    .unwrap();
    let mut fotolife = Fotolife::new(oauth);

    fotolife.get_image("hogehoge").unwrap();
  }
}
