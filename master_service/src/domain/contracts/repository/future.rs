use std::{future::Future, pin::Pin};

use crate::domain::models::error::RepositoryError;

/// A repository operation future that can be used through trait objects.
pub type RepositoryFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, RepositoryError>> + Send + 'a>>;
