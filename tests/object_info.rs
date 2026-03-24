use fujinx::ObjectInfo;

#[test]
fn roundtrip() {
    let original = ObjectInfo {
        format_code: 0x3801,
        compressed_size: 1_234_567,
        filename: "DSCF1234.RAF".to_string(),
    };

    let data = original.to_dataset();
    let parsed = ObjectInfo::parse(&data).unwrap();

    assert_eq!(parsed.format_code, 0x3801);
    assert_eq!(parsed.compressed_size, 1_234_567);
    assert_eq!(parsed.filename, "DSCF1234.RAF");
}

#[test]
fn parse_truncated_data_fails() {
    let data = ObjectInfo {
        format_code: 0x3801,
        compressed_size: 100,
        filename: "test.jpg".to_string(),
    }
    .to_dataset();

    assert!(ObjectInfo::parse(&data[..10]).is_err());
}
