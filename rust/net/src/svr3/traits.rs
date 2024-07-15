//
// Copyright 2024 Signal Messenger, LLC.
// SPDX-License-Identifier: AGPL-3.0-only
//

//! Most of the traits in this module are likely to be used together
//! therefore the module exists as a sort of a "prelude" to make importing them
//! all in bulk easier.

use std::num::NonZeroU32;

use async_trait::async_trait;
use rand_core::CryptoRngCore;

use libsignal_svr3::EvaluationResult;

use crate::enclave;
use crate::enclave::PpssSetup;
use crate::infra::AsyncDuplexStream;

use super::{ppss_ops, Error, OpaqueMaskedShareSet};

#[async_trait]
pub trait Backup {
    async fn backup(
        &self,
        password: &str,
        secret: [u8; 32],
        max_tries: NonZeroU32,
        rng: &mut (impl CryptoRngCore + Send),
    ) -> Result<OpaqueMaskedShareSet, Error>;
}

#[async_trait]
pub trait Restore {
    async fn restore(
        &self,
        password: &str,
        share_set: OpaqueMaskedShareSet,
        rng: &mut (impl CryptoRngCore + Send),
    ) -> Result<EvaluationResult, Error>;
}

#[async_trait]
pub trait Query {
    async fn query(&self) -> Result<u32, Error>;
}

#[async_trait]
pub trait Remove {
    async fn remove(&self) -> Result<(), Error>;
}

#[async_trait]
pub trait Svr3Connect {
    // Stream is needed for the blanket implementation,
    // otherwise S would be an unconstrained generic parameter.
    type Stream;
    type Env: PpssSetup<Self::Stream>;
    async fn connect(
        &self,
    ) -> Result<<Self::Env as PpssSetup<Self::Stream>>::Connections, enclave::Error>;
}

#[async_trait]
impl<T> Backup for T
where
    T: Svr3Connect + Sync,
    T::Stream: AsyncDuplexStream + 'static,
{
    async fn backup(
        &self,
        password: &str,
        secret: [u8; 32],
        max_tries: NonZeroU32,
        rng: &mut (impl CryptoRngCore + Send),
    ) -> Result<OpaqueMaskedShareSet, Error> {
        ppss_ops::do_backup::<T::Stream, T::Env>(
            self.connect().await?,
            password,
            secret,
            max_tries,
            rng,
        )
        .await
    }
}

#[async_trait]
impl<T> Restore for T
where
    T: Svr3Connect + Sync,
    T::Stream: AsyncDuplexStream + 'static,
{
    async fn restore(
        &self,
        password: &str,
        share_set: OpaqueMaskedShareSet,
        rng: &mut (impl CryptoRngCore + Send),
    ) -> Result<EvaluationResult, Error> {
        ppss_ops::do_restore::<T::Stream, T::Env>(self.connect().await?, password, share_set, rng)
            .await
    }
}

#[async_trait]
impl<T> Remove for T
where
    T: Svr3Connect + Sync,
    T::Stream: AsyncDuplexStream + 'static,
{
    async fn remove(&self) -> Result<(), Error> {
        ppss_ops::do_remove::<T::Stream, T::Env>(self.connect().await?).await
    }
}

#[async_trait]
impl<T> Query for T
where
    T: Svr3Connect + Sync,
    T::Stream: AsyncDuplexStream + 'static,
{
    async fn query(&self) -> Result<u32, Error> {
        ppss_ops::do_query::<T::Stream, T::Env>(self.connect().await?).await
    }
}
