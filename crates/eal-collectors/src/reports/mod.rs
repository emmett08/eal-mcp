//! Pinned report imports with original measurement time and explicit asserted provenance.
mod junit;
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct ReportConfig {
    pub manifest: PathBuf,
    pub sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: String,
    repository: String,
    revision: String,
    observed_at: String,
    format: Format,
    artifact: PathBuf,
    artifact_sha256: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Format {
    Junit,
    Sarif,
    Json,
}
impl ReportConfig {
    pub(crate) fn validate(&self) -> Result<(), String> {
        crate::config::absolute(&self.manifest)?;
        self.load().map(|_| ())
    }
    fn load(&self) -> Result<Manifest, String> {
        let bytes = crate::api::config::pinned(&self.manifest, &self.sha256)?;
        let manifest: Manifest = serde_json::from_value(
            eal_adapters::parse_json(&bytes).map_err(|_| "Report manifest requires strict JSON")?,
        )
        .map_err(|_| "Invalid report manifest")?;
        if manifest.schema != "eal.report-source/1"
            || manifest.repository.trim().is_empty()
            || manifest.repository.len() > 1024
            || manifest.repository.chars().any(char::is_control)
            || manifest.revision.len() != 40
            || !manifest.revision.bytes().all(|b| b.is_ascii_hexdigit())
            || !crate::api::config::digest(&manifest.artifact_sha256)
        {
            return Err(
                "Report manifest lacks bounded provenance and exact revision/digest".into(),
            );
        }
        crate::config::absolute(&manifest.artifact)?;
        eal_adapters::retained::timestamp(&manifest.observed_at)
            .map_err(|_| "Report requires its original UTC measurement time")?;
        Ok(manifest)
    }
}
pub(crate) fn collect(
    config: &ReportConfig,
    request: &crate::Request,
) -> Result<(Value, String), String> {
    request.fields(&["revision", "report_sha256"])?;
    let manifest = config.load()?;
    request.matches("revision", &manifest.revision)?;
    request.matches("report_sha256", &manifest.artifact_sha256)?;
    let bytes = crate::api::config::pinned(&manifest.artifact, &manifest.artifact_sha256)?;
    let (summary, report, format) = match manifest.format {
        Format::Junit => {
            let summary = junit::summarise(&bytes)?;
            let xml = std::str::from_utf8(&bytes).map_err(|_| "JUnit requires UTF-8 XML")?;
            (summary, json!(xml), "junit")
        }
        Format::Sarif => {
            let value =
                eal_adapters::parse_json(&bytes).map_err(|_| "SARIF requires strict JSON")?;
            if value["version"] != "2.1.0" {
                return Err("SARIF requires version 2.1.0".into());
            }
            let runs = value["runs"]
                .as_array()
                .ok_or("SARIF requires explicit runs")?;
            let mut findings = 0usize;
            for run in runs {
                if run["tool"]["driver"]["name"].as_str().is_none() {
                    return Err("SARIF run lacks tool identity".into());
                }
                findings = findings
                    .checked_add(
                        run["results"]
                            .as_array()
                            .ok_or("SARIF profile requires explicit results")?
                            .len(),
                    )
                    .ok_or("SARIF finding count overflow")?;
            }
            (
                json!({"run_count":runs.len(),"finding_count":findings,"coverage":"unspecified"}),
                value,
                "sarif",
            )
        }
        Format::Json => {
            let value = eal_adapters::parse_json(&bytes)
                .map_err(|_| "Retained report requires strict JSON")?;
            if !value.is_object() {
                return Err("Retained JSON report requires an object".into());
            }
            (json!({"coverage":"unspecified"}), value, "json")
        }
    };
    Ok((
        json!({"collected":true,"measurement_only":true,"repository":manifest.repository,"revision":manifest.revision,"artifact_sha256":manifest.artifact_sha256,
        "manifest_sha256":config.sha256,"provenance":"operator_asserted_revision_correspondence","format":format,"summary":summary,"report":report}),
        manifest.observed_at,
    ))
}
