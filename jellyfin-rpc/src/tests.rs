use crate::ClientBuilder;

#[test]
fn build_client_error() {
    let client = ClientBuilder::new().build();

    if let Ok(_) = client {
        panic!("client was constructed even though required values are missing!");
    }
}

#[test]
fn invalid_url() {
    let mut builder = ClientBuilder::new();
    builder
        .api_key("a1b2c3d4")
        .username("test")
        .url("url_without_base.com");

    let client = builder.build();

    if let Ok(_) = client {
        panic!("client constructed without a valid url!")
    }
}

#[test]
fn base64_encoding() {
    use crate::external::image_utils::base64_encode;
    assert_eq!(base64_encode(b""), "");
    assert_eq!(base64_encode(b"f"), "Zg==");
    assert_eq!(base64_encode(b"fo"), "Zm8=");
    assert_eq!(base64_encode(b"foo"), "Zm9v");
}

#[test]
fn config_roundtrip() {
    use crate::config::ConfigBuilder;
    let builder = ConfigBuilder::default();
    let json = serde_json::to_string(&builder).unwrap();
    let parsed: ConfigBuilder = serde_json::from_str(&json).unwrap();
    assert_eq!(builder, parsed);
}

#[test]
fn image_processing_makes_square_png() {
    use crate::external::image_utils::{make_square_with_blur, ImageProcessingOptions};

    let source = image::RgbImage::from_pixel(60, 30, image::Rgb([200, 40, 40]));
    let mut bytes = Vec::new();
    source
        .write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)
        .unwrap();

    for background in [true, false] {
        let options = ImageProcessingOptions {
            size: Some(64),
            background,
            ..Default::default()
        };
        let out = make_square_with_blur(&bytes, &options).unwrap();
        let img = image::load_from_memory(&out).unwrap();
        assert_eq!((img.width(), img.height()), (64, 64));
    }
}
