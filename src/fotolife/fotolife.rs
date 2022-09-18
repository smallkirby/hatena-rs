/// Response from Fotolife POST API
#[derive(Debug)]
pub struct FotolifePostResponse {
  pub image_id: String, // ID of uploaded image
}

impl FotolifePostResponse {
  pub fn new(image_id: String) -> Self {
    Self { image_id }
  }
}
