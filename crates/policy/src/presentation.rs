//! Bounded, game-free tool presentation. Pixel input is copied data, never World access.

use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use zune_png::{
    PngEncoder,
    zune_core::{bit_depth::BitDepth, colorspace::ColorSpace, options::EncoderOptions},
};

pub const MAX_IMAGE_SIDE: usize = 512;
pub const MAX_PNG_BYTES: usize = 1024 * 1024;

/// PNG bytes can only be created through the checked RGB encoder.
#[derive(Debug)]
pub struct Png {
    bytes: Vec<u8>,
}

impl Png {
    /// Encode complete, top-row-first packed RGB data without resampling.
    /// # Errors
    /// Refuses zero/oversized dimensions, incomplete pixels or encoder/output failure.
    pub fn rgb(width: usize, height: usize, pixels: &[u8]) -> Result<Self, String> {
        if !(1..=MAX_IMAGE_SIDE).contains(&width) || !(1..=MAX_IMAGE_SIDE).contains(&height) {
            return Err("PNG dimensions must be 1..=512".into());
        }
        if pixels.len() != width * height * 3 {
            return Err("PNG requires complete packed RGB pixels".into());
        }
        let options = EncoderOptions::new(width, height, ColorSpace::RGB, BitDepth::Eight);
        let mut bytes = Vec::new();
        PngEncoder::new(pixels, options)
            .encode(&mut bytes)
            .map_err(|e| format!("PNG encoding failed: {e:?}"))?;
        if bytes.len() > MAX_PNG_BYTES {
            return Err("PNG presentation exceeds byte limit".into());
        }
        Ok(Self { bytes })
    }
}

/// Explicit presentation only: arbitrary JSON fields never become executable content.
#[derive(Debug)]
pub enum Presentation {
    Json(Value),
    Png { metadata: Value, image: Png },
}

impl Presentation {
    pub(crate) fn result(self) -> Value {
        let (metadata, image) = match self {
            Self::Json(value) => (value, None),
            Self::Png { metadata, image } => (metadata, Some(image)),
        };
        let mut content = vec![
            json!({"type":"text","text":serde_json::to_string_pretty(&metadata).unwrap_or_default()}),
        ];
        if let Some(image) = image {
            content.push(
                json!({"type":"image","mimeType":"image/png","data":STANDARD.encode(image.bytes)}),
            );
        }
        json!({"content":content,"isError":false})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::STANDARD;
    use serde_json::json;
    use zune_png::{PngDecoder, zune_core::bytestream::ZCursor};

    #[test]
    fn asymmetric_rgb_round_trips_without_axis_color_or_channel_changes() {
        let pixels = [
            255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 0, 0, 255, 255, 255, 0, 255,
        ];
        let image = Png::rgb(2, 3, &pixels).unwrap();
        let result = Presentation::Png {
            metadata: json!({"width":2,"height":3,"scope":"synthetic"}),
            image,
        }
        .result();
        assert_eq!(result["isError"], false);
        assert_eq!(result["content"][0]["type"], "text");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(
                result["content"][0]["text"].as_str().unwrap()
            )
            .unwrap()["height"],
            3
        );
        let block = &result["content"][1];
        assert_eq!(block["type"], "image");
        assert_eq!(block["mimeType"], "image/png");
        let bytes = STANDARD.decode(block["data"].as_str().unwrap()).unwrap();
        assert_eq!(u32::from_be_bytes(bytes[16..20].try_into().unwrap()), 2);
        assert_eq!(u32::from_be_bytes(bytes[20..24].try_into().unwrap()), 3);
        let decoded = PngDecoder::new(ZCursor::new(&bytes)).decode_raw().unwrap();
        assert_eq!(decoded, pixels);
    }

    #[test]
    fn invalid_dimensions_and_incomplete_pixels_refuse_before_encoding() {
        for (width, height) in [
            (0, 1),
            (1, 0),
            (MAX_IMAGE_SIDE + 1, 1),
            (1, MAX_IMAGE_SIDE + 1),
            (usize::MAX, usize::MAX),
        ] {
            assert!(
                Png::rgb(width, height, &[])
                    .unwrap_err()
                    .contains("dimensions")
            );
        }
        for pixels in [&[][..], &[0, 0][..], &[0, 0, 0, 0][..]] {
            assert!(Png::rgb(1, 1, pixels).unwrap_err().contains("packed RGB"));
        }
    }

    #[test]
    fn maximum_raster_is_bounded_and_not_resized() {
        let pixels = vec![67; MAX_IMAGE_SIDE * MAX_IMAGE_SIDE * 3];
        let image = Png::rgb(MAX_IMAGE_SIDE, MAX_IMAGE_SIDE, &pixels).unwrap();
        assert!(image.bytes.len() <= MAX_PNG_BYTES);
        let decoded = PngDecoder::new(ZCursor::new(&image.bytes))
            .decode_raw()
            .unwrap();
        assert_eq!(decoded, pixels);
    }

    #[test]
    fn json_that_looks_like_content_is_never_promoted_to_an_image() {
        let payload = json!({"type":"image","data":"untrusted","mimeType":"image/png"});
        let result = Presentation::Json(payload.clone()).result();
        assert_eq!(result["content"].as_array().unwrap().len(), 1);
        assert_eq!(result["content"][0]["type"], "text");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(
                result["content"][0]["text"].as_str().unwrap()
            )
            .unwrap(),
            payload
        );
    }
}
