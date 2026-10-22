use std::sync::Arc;

use crate::domain::{
    contracts::repository::IndonesiaBoundaryRepository,
    models::AppContext,
    usecases::{UsecaseFuture, geographic::IsInsideIndonesia},
};

pub struct IndonesiaQueries {
    boundary: Arc<dyn IndonesiaBoundaryRepository>,
}

impl IndonesiaQueries {
    pub fn new(boundary: Arc<dyn IndonesiaBoundaryRepository>) -> Self {
        Self { boundary }
    }
}

impl IsInsideIndonesia for IndonesiaQueries {
    fn is_inside_indonesia<'a>(
        &'a self,
        context: &'a AppContext,
        longitude: f64,
        latitude: f64,
    ) -> UsecaseFuture<'a, bool> {
        Box::pin(async move {
            self.boundary
                .is_inside(context, longitude, latitude)
                .await
                .map_err(Into::into)
        })
    }
}
