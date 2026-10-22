use std::{future::Future, pin::Pin};

use crate::domain::models::UsecaseError;

/// A usecase operation future that can be used through trait objects.
pub type UsecaseFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, UsecaseError>> + Send + 'a>>;
