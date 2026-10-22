use master_service::domain::models::entity::{Boundary, BoundaryError, CityKind};

fn ring() -> Vec<[f64; 2]> {
    vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0]]
}

#[test]
fn valid_multipolygon_is_retained() {
    let coordinates = vec![vec![ring()]];
    let boundary = Boundary::new(coordinates.clone()).unwrap();

    assert_eq!(boundary.coordinates(), coordinates.as_slice());
}

#[test]
fn empty_and_malformed_shapes_are_rejected() {
    assert_eq!(Boundary::new(vec![]).unwrap_err(), BoundaryError::Empty);
    assert_eq!(
        Boundary::new(vec![vec![]]).unwrap_err(),
        BoundaryError::EmptyPolygon
    );
    assert_eq!(
        Boundary::new(vec![vec![vec![[0.0, 0.0], [1.0, 0.0], [0.0, 0.0]]]]).unwrap_err(),
        BoundaryError::RingTooShort
    );
    assert_eq!(
        Boundary::new(vec![vec![vec![
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
        ]]])
        .unwrap_err(),
        BoundaryError::UnclosedRing
    );
}

#[test]
fn invalid_coordinates_are_rejected() {
    assert_eq!(
        Boundary::new(vec![vec![vec![
            [0.0, 0.0],
            [f64::NAN, 0.0],
            [1.0, 1.0],
            [0.0, 0.0],
        ]]])
        .unwrap_err(),
        BoundaryError::NonFiniteCoordinate
    );
    assert_eq!(
        Boundary::new(vec![vec![vec![
            [181.0, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [181.0, 0.0],
        ]]])
        .unwrap_err(),
        BoundaryError::LongitudeOutOfRange
    );
    assert_eq!(
        Boundary::new(vec![vec![vec![
            [0.0, 91.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 91.0],
        ]]])
        .unwrap_err(),
        BoundaryError::LatitudeOutOfRange
    );
}

#[test]
fn city_kind_uses_database_spelling() {
    assert_eq!(CityKind::City.as_str(), "city");
    assert_eq!(CityKind::Regency.as_str(), "regency");
}
