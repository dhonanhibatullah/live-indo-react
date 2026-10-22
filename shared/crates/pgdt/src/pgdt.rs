use std::sync::Arc;

use futures_core::stream::BoxStream;
use futures_util::TryStreamExt;
use sqlx::{
    Execute, Executor, PgPool, Postgres,
    postgres::{PgQueryResult, PgRow},
};

use crate::{Transaction, TransactionContext, Transactor, transaction::inactive};

/// Clone this wrapper to share its transaction identity with repositories and transactors.
#[derive(Clone, Debug)]
pub struct Pgdt {
    pool: PgPool,
    owner: Arc<()>,
}

impl Pgdt {
    /// Independently created wrappers have different identities, even for the same pool.
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            owner: Arc::new(()),
        }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
    pub fn transactor(&self) -> Transactor {
        Transactor::new(self.clone())
    }

    pub(crate) fn transaction<C: TransactionContext>(
        &self,
        context: &C,
    ) -> Result<Option<Transaction>, sqlx::Error> {
        let Some(transaction) = context.transaction() else {
            return Ok(None);
        };
        if !Arc::ptr_eq(&transaction.state.owner, &self.owner) {
            return Err(sqlx::Error::Protocol(
                "transaction belongs to another Pgdt wrapper".into(),
            ));
        }
        if !transaction.is_active() {
            return Err(inactive());
        }
        Ok(Some(transaction.clone()))
    }

    pub(crate) fn owner(&self) -> Arc<()> {
        self.owner.clone()
    }

    pub async fn exec<'q, C, Q>(&self, context: &C, query: Q) -> Result<PgQueryResult, sqlx::Error>
    where
        C: TransactionContext,
        Q: Execute<'q, Postgres> + 'q,
    {
        if let Some(transaction) = self.transaction(context)? {
            let mut guard = transaction.state.transaction.lock().await;
            if !transaction.is_active() {
                return Err(inactive());
            }
            let tx = guard.as_mut().ok_or_else(inactive)?;
            (&mut **tx).execute(query).await
        } else {
            (&self.pool).execute(query).await
        }
    }

    /// Streams rows lazily. Consume or drop the stream before the transaction callback
    /// completes or before awaiting another operation on that transaction.
    pub fn query<'a, 'q: 'a, C, Q>(
        &'a self,
        context: &'a C,
        query: Q,
    ) -> BoxStream<'a, Result<PgRow, sqlx::Error>>
    where
        C: TransactionContext,
        Q: Execute<'q, Postgres> + 'q,
    {
        Box::pin(async_stream::try_stream! {
            if let Some(transaction) = self.transaction(context)? {
                let mut guard = transaction.state.transaction.lock().await;
                if !transaction.is_active() { Err(inactive())?; }
                let tx = guard.as_mut().ok_or_else(inactive)?;
                let mut rows = (&mut **tx).fetch(query);
                while let Some(row) = rows.try_next().await? { yield row; }
            } else {
                let mut rows = (&self.pool).fetch(query);
                while let Some(row) = rows.try_next().await? { yield row; }
            }
        })
    }

    pub async fn query_row<'q, C, Q>(&self, context: &C, query: Q) -> Result<PgRow, sqlx::Error>
    where
        C: TransactionContext,
        Q: Execute<'q, Postgres> + 'q,
    {
        self.query_optional(context, query)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn query_optional<'q, C, Q>(
        &self,
        context: &C,
        query: Q,
    ) -> Result<Option<PgRow>, sqlx::Error>
    where
        C: TransactionContext,
        Q: Execute<'q, Postgres> + 'q,
    {
        if let Some(transaction) = self.transaction(context)? {
            let mut guard = transaction.state.transaction.lock().await;
            if !transaction.is_active() {
                return Err(inactive());
            }
            let tx = guard.as_mut().ok_or_else(inactive)?;
            (&mut **tx).fetch_optional(query).await
        } else {
            (&self.pool).fetch_optional(query).await
        }
    }

    /// Equivalent to `self.transactor().with_tx(context, operation)`.
    pub async fn with_tx<C, F, Fut, T, E>(&self, context: &C, operation: F) -> Result<T, E>
    where
        C: TransactionContext,
        F: FnOnce(C) -> Fut + Send,
        Fut: std::future::Future<Output = Result<T, E>> + Send,
        T: Send,
        E: From<sqlx::Error> + Send,
    {
        self.transactor().with_tx(context, operation).await
    }
}
