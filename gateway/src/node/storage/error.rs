// Copyright 2021 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum StorageError {
    #[error("Database experienced an internal error: {0}")]
    InternalDatabaseError(#[from] sqlx::Error),

    #[error("Failed to perform database migration: {0}")]
    MigrationError(#[from] sqlx::migrate::MigrateError),
}
