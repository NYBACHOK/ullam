use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FlightAnalysis {
    pub termination: TerminationAnalysis,
    pub hypotheses: Vec<Hypothesis>,
    pub issues: Vec<Issue>,
    pub evidence_gaps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TerminationAnalysis {
    /// What was directly observed, e.g. "EV=11 DISARMED".
    pub event: String,
    /// Evidence-based explanation.
    pub assessment: String,
    /// Confidence in the explanation, not statistical probability.
    pub confidence: u8,
    pub causal_chain: Vec<String>,
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Hypothesis {
    pub hypothesis: String,
    /// Confidence in this hypothesis based on available evidence.
    pub confidence: u8,
    pub supporting_evidence: Vec<Evidence>,
    pub contradicting_evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Evidence {
    pub claim: String,
    pub source: EvidenceSource,
    pub timestamp: Option<f64>,
    pub strength: EvidenceStrength,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceSource {
    pub message: String,
    pub field: Option<String>,
    pub timestamp: Option<f64>,
    pub value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStrength {
    Strong,
    Moderate,
    Weak,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Issue {
    pub issue: String,
    pub importance: IssueImportance,
    pub description: String,
    pub evidence: Vec<Evidence>,
    pub recommended_investigation: Option<String>,
    pub causal_relevance: Option<CausalRelevance>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IssueImportance {
    Critical,
    Important,
    Minor,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CausalRelevance {
    LikelyCause,
    PossibleCause,
    ContributingFactor,
    Unrelated,
    Unknown,
}
