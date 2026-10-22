use master_service::domain::{
    contracts::utility::AppContextFactory, usecases::misc::app_context::CreateAppContext,
};

#[test]
fn app_context_contracts_are_object_safe() {
    fn accept(_: &dyn AppContextFactory, _: &dyn CreateAppContext) {}

    let _ = accept;
}
