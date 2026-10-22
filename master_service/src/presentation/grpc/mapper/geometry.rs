use live_indo_proto::live_indo_react::master::v1 as proto;
use tonic::Status;

use crate::domain::models::entity::Boundary;

use super::error::bad_args;

pub(crate) fn multi_polygon(value: Boundary) -> proto::MultiPolygon {
    proto::MultiPolygon {
        polygons: value
            .coordinates()
            .iter()
            .map(|polygon| proto::Polygon {
                rings: polygon
                    .iter()
                    .map(|ring| proto::Ring {
                        positions: ring
                            .iter()
                            .map(|position| proto::Coordinate {
                                longitude: position[0],
                                latitude: position[1],
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect(),
    }
}

pub(crate) fn parse_boundary(value: proto::MultiPolygon) -> Result<Boundary, Status> {
    let coordinates = value
        .polygons
        .into_iter()
        .map(|polygon| {
            polygon
                .rings
                .into_iter()
                .map(|ring| {
                    ring.positions
                        .into_iter()
                        .map(|position| [position.longitude, position.latitude])
                        .collect()
                })
                .collect()
        })
        .collect();

    Boundary::new(coordinates).map_err(|_| bad_args("boundary is structurally invalid"))
}

pub(crate) fn required_boundary(value: Option<proto::MultiPolygon>) -> Result<Boundary, Status> {
    parse_boundary(value.ok_or_else(|| bad_args("boundary is required"))?)
}

pub(crate) fn optional_boundary(
    value: Option<proto::MultiPolygon>,
) -> Result<Option<Boundary>, Status> {
    value.map(parse_boundary).transpose()
}
