//! Integration tests for Thai number-to-words.

use sea_g2p_rs::lang::th::num2th::{n2w, n2w_decimal, n2w_single};
#[test]
fn tens_alternations() {
    assert_eq!(n2w("10"), "สิบ");         // not หนึ่งสิบ
    assert_eq!(n2w("15"), "สิบห้า");
    assert_eq!(n2w("20"), "ยี่สิบ");       // not สองสิบ
    assert_eq!(n2w("21"), "ยี่สิบเอ็ด");   // final 1 -> เอ็ด
    assert_eq!(n2w("31"), "สามสิบเอ็ด");
    assert_eq!(n2w("101"), "หนึ่งร้อยเอ็ด");
}

#[test]
fn cardinals() {
    assert_eq!(n2w("0"), "ศูนย์");
    assert_eq!(n2w("7"), "เจ็ด");
    assert_eq!(n2w("100"), "หนึ่งร้อย");
    assert_eq!(n2w("1250"), "หนึ่งพันสองร้อยห้าสิบ");
    assert_eq!(n2w("2560"), "สองพันห้าร้อยหกสิบ");
    assert_eq!(n2w("1000000"), "หนึ่งล้าน");
}

#[test]
fn digit_by_digit_and_decimals() {
    assert_eq!(n2w_single("081"), "ศูนย์ แปด หนึ่ง");
    assert_eq!(n2w_decimal("3", "14"), "สามจุดหนึ่งสี่");
}
