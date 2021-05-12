#![cfg(test)]
use dirt::args::fraction::{ConversionError::*, Fraction, FromStrError::*};
use pretty_assertions::assert_eq;

#[test]
#[allow(clippy::float_cmp)]
fn typical() {
    let actual: f32 = "0.5".parse::<Fraction>().expect("create ratio").into();
    assert_eq!(actual, 0.5);
}

#[test]
#[allow(clippy::float_cmp)]
fn equal_to_zero() {
    let actual: f32 = "0".parse::<Fraction>().expect("create ratio").into();
    assert_eq!(actual, 0.0);
}

#[test]
fn less_than_zero() {
    let actual_error = "-0.1".parse::<Fraction>().expect_err("cause bound error");
    let actual_message = actual_error.to_string();
    let expected_error = Conversion(LowerBound);
    let expected_message = "less than 0".to_string();
    assert_eq!(
        (actual_error, actual_message),
        (expected_error, expected_message),
    );
}

#[test]
#[allow(clippy::float_cmp)]
fn less_than_one() {
    let actual: f32 = "0.99999".parse::<Fraction>().expect("create ratio").into();
    assert_eq!(actual, 0.99999);
}

#[test]
fn equal_to_one() {
    let actual_error = "1".parse::<Fraction>().expect_err("cause bound error");
    let actual_message = actual_error.to_string();
    let expected_error = Conversion(UpperBound);
    let expected_message = "greater than or equal to 1".to_string();
    assert_eq!(
        (actual_error, actual_message),
        (expected_error, expected_message),
    );
}

#[test]
fn invalid_float_literal() {
    let actual = "a"
        .parse::<Fraction>()
        .expect_err("cause syntax error")
        .to_string();
    let expected = "invalid float literal";
    assert_eq!(actual, expected);
}
