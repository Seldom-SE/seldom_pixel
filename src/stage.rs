//! Stages used by this crate

use crate::prelude::*;

/// Stages used by this crate
#[derive(Debug, StageLabel)]
pub enum PxStage {
    /// Runs before the [`CoreStage::Update`] stage
    PreUpdate,
    /// Runs before the [`PxStage::PostUpdate`] stage
    PrePostUpdate,
    /// Runs after the [`CoreStage::Update`] stage
    PostUpdate,
    /// Runs after the [`PxStage::PostUpdate`] stage
    Last,
}
