use super::*;
use crate::models::Boundary;

#[test]
fn city_prefix_must_match_its_province_before_database_execution() {
    let mut city = City {
        kemendagri_code: "11.01".into(),
        province_kemendagri_code: "11".into(),
        kind: CityKind::Regency,
        name: "Test".into(),
        boundary: Boundary::MultiPolygon {
            coordinates: vec![],
        },
        boundary_source: "fixture".into(),
        boundary_reference_date: None,
    };
    validate_parent(&city).unwrap();
    city.province_kemendagri_code = "12".into();
    let error = validate_parent(&city).unwrap_err();
    assert_eq!(error.code.as_deref(), Some("11.01"));
    city.kemendagri_code = "1101".into();
    assert!(validate_parent(&city).is_err());
}
