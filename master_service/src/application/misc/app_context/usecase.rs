use std::sync::Arc;

use uuid::Uuid;

use crate::domain::{
    contracts::utility::app_context::AppContextFactory, models::AppContext,
    usecases::misc::app_context::CreateAppContext,
};

pub struct AppContextService {
    factory: Arc<dyn AppContextFactory>,
}

impl AppContextService {
    pub fn new(factory: Arc<dyn AppContextFactory>) -> Self {
        Self { factory }
    }
}

impl CreateAppContext for AppContextService {
    fn create_app_context(&self, actor: Option<String>, trace_id: Option<Uuid>) -> AppContext {
        self.factory.create(actor, trace_id)
    }
}
