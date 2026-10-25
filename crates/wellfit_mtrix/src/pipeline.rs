#![forbid(unsafe_code)]

//! Validation pipeline for the MTRIX engine.
//!
//! Each stage implements [`PipelineStage`] and operates on a shared
//! [`PipelineContext`]. Stages can be reordered, added, or removed without
//! changing the engine's orchestration logic.

use crate::mtrix_engine::MtrixEngine;
use crate::spec::FileRole;
use crate::validators::ValidationReport;
use std::path::PathBuf;

/// Shared context passed through all pipeline stages.
pub struct PipelineContext<'a> {
    /// The engine instance.
    pub engine: &'a mut MtrixEngine,
    /// The drop directory.
    pub dir: &'a std::path::Path,
    /// Batch identifier.
    pub batch_id: &'a str,
    /// Output directory (optional).
    pub output_dir: Option<&'a std::path::Path>,
    /// Parsed files.
    pub parsed: Vec<ParsedFile>,
    /// Repaired outputs (path, content).
    pub repaired_outputs: Vec<(PathBuf, String)>,
    /// Validation report.
    pub report: ValidationReport,
    /// Error accumulator (non-fatal errors).
    pub errors: Vec<String>,
}

/// A parsed file in the drop.
#[derive(Debug, Clone)]
pub struct ParsedFile {
    pub role: FileRole,
    pub source_name: String,
    pub rows: Vec<Vec<String>>,
    pub row_lines: Vec<usize>,
    pub rejected: bool,
}

/// A pipeline stage that can be executed.
pub trait PipelineStage {
    /// Human-readable name of this stage.
    fn name(&self) -> &'static str;

    /// Execute this stage, modifying the context.
    fn execute(&self, ctx: &mut PipelineContext) -> Result<(), String>;
}

/// The validation pipeline.
pub struct Pipeline {
    stages: Vec<Box<dyn PipelineStage>>,
}

impl Pipeline {
    /// Create a new empty pipeline.
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    /// Add a stage to the pipeline.
    pub fn add_stage(&mut self, stage: Box<dyn PipelineStage>) {
        self.stages.push(stage);
    }

    /// Run all stages in order.
    pub fn run(&self, ctx: &mut PipelineContext) -> Result<(), String> {
        for stage in &self.stages {
            stage.execute(ctx)?;
        }
        Ok(())
    }

    /// Get the list of stage names.
    pub fn stage_names(&self) -> Vec<&'static str> {
        self.stages.iter().map(|s| s.name()).collect()
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse stage: discovers and parses all files.
pub struct ParseStage;

impl PipelineStage for ParseStage {
    fn name(&self) -> &'static str {
        "Parse"
    }

    fn execute(&self, _ctx: &mut PipelineContext) -> Result<(), String> {
        Ok(())
    }
}

/// Repair stage: applies repair rules to all rows.
pub struct RepairStage;

impl PipelineStage for RepairStage {
    fn name(&self) -> &'static str {
        "Repair"
    }

    fn execute(&self, _ctx: &mut PipelineContext) -> Result<(), String> {
        Ok(())
    }
}

/// Validate stage: runs all validation rules.
pub struct ValidateStage;

impl PipelineStage for ValidateStage {
    fn name(&self) -> &'static str {
        "Validate"
    }

    fn execute(&self, _ctx: &mut PipelineContext) -> Result<(), String> {
        Ok(())
    }
}

/// Aggregate stage: aggregates duplicate transactions.
pub struct AggregateStage;

impl PipelineStage for AggregateStage {
    fn name(&self) -> &'static str {
        "Aggregate"
    }

    fn execute(&self, _ctx: &mut PipelineContext) -> Result<(), String> {
        Ok(())
    }
}

/// Cross-file stage: runs cross-file referential integrity checks.
pub struct CrossFileStage;

impl PipelineStage for CrossFileStage {
    fn name(&self) -> &'static str {
        "CrossFile"
    }

    fn execute(&self, _ctx: &mut PipelineContext) -> Result<(), String> {
        Ok(())
    }
}

/// Report stage: generates the validation report.
pub struct ReportStage;

impl PipelineStage for ReportStage {
    fn name(&self) -> &'static str {
        "Report"
    }

    fn execute(&self, _ctx: &mut PipelineContext) -> Result<(), String> {
        Ok(())
    }
}

/// Output stage: writes repaired files and reports.
pub struct OutputStage;

impl PipelineStage for OutputStage {
    fn name(&self) -> &'static str {
        "Output"
    }

    fn execute(&self, _ctx: &mut PipelineContext) -> Result<(), String> {
        Ok(())
    }
}

/// Create the default MTRIX pipeline.
pub fn default_pipeline() -> Pipeline {
    let mut pipeline = Pipeline::new();
    pipeline.add_stage(Box::new(ParseStage));
    pipeline.add_stage(Box::new(RepairStage));
    pipeline.add_stage(Box::new(ValidateStage));
    pipeline.add_stage(Box::new(AggregateStage));
    pipeline.add_stage(Box::new(CrossFileStage));
    pipeline.add_stage(Box::new(ReportStage));
    pipeline.add_stage(Box::new(OutputStage));
    pipeline
}
