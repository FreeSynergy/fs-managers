use fs_manager_language::{DateFormat, Language, NumberFormat, TimeFormat};

#[test]
fn language_from_code_german_has_id() {
    let lang = Language::from_code("de");
    assert_eq!(lang.id, "de");
}

#[test]
fn language_from_code_unknown_preserves_id() {
    let lang = Language::from_code("xyz");
    assert_eq!(lang.id, "xyz");
}

#[test]
fn date_format_ymd_formats_correctly() {
    let formatted = DateFormat::Ymd.format(2026, 3, 30);
    assert_eq!(formatted, "2026-03-30");
}

#[test]
fn date_format_dmy_formats_correctly() {
    let formatted = DateFormat::DmY.format(2026, 3, 30);
    assert_eq!(formatted, "30.03.2026");
}

#[test]
fn date_format_mdy_formats_correctly() {
    let formatted = DateFormat::MdY.format(2026, 3, 30);
    assert_eq!(formatted, "03/30/2026");
}

#[test]
fn time_format_24h_formats_correctly() {
    let formatted = TimeFormat::H24.format(14, 5);
    assert_eq!(formatted, "14:05");
}

#[test]
fn time_format_12h_includes_hour() {
    let formatted = TimeFormat::H12.format(14, 5);
    assert!(formatted.contains("2") && formatted.contains("05"));
}

#[test]
fn number_format_integer_europe_dot() {
    let n = NumberFormat::EuropeDot.format_integer(1_000_000);
    assert_eq!(n, "1.000.000");
}

#[test]
fn number_format_integer_us_comma() {
    let n = NumberFormat::UsComma.format_integer(1_000_000);
    assert_eq!(n, "1,000,000");
}

#[test]
fn number_format_integer_negative() {
    let n = NumberFormat::UsComma.format_integer(-42);
    assert!(n.starts_with('-'));
}
