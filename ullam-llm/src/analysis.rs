use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use std::fmt::{self, Display, Formatter};

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

impl Display for FlightAnalysis {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Flight Analysis")?;
        writeln!(f, "===============")?;

        writeln!(f)?;
        writeln!(f, "Termination Analysis")?;
        writeln!(f, "--------------------")?;
        writeln!(f, "Event: {}", self.termination.event)?;
        writeln!(f, "Assessment: {}", self.termination.assessment)?;
        writeln!(f, "Confidence: {}%", self.termination.confidence)?;

        if !self.termination.causal_chain.is_empty() {
            writeln!(f)?;
            writeln!(f, "Causal Chain:")?;

            for (i, step) in self.termination.causal_chain.iter().enumerate() {
                writeln!(f, "  {}. {}", i + 1, step)?;
            }
        }

        if !self.termination.evidence.is_empty() {
            writeln!(f)?;
            writeln!(f, "Evidence:")?;

            for evidence in &self.termination.evidence {
                writeln!(f, "{evidence}")?;
            }
        }

        if !self.hypotheses.is_empty() {
            writeln!(f)?;
            writeln!(f, "Competing Hypotheses")?;
            writeln!(f, "--------------------")?;

            for (i, hypothesis) in self.hypotheses.iter().enumerate() {
                writeln!(
                    f,
                    "{}. {} ({}% confidence)",
                    i + 1,
                    hypothesis.hypothesis,
                    hypothesis.confidence
                )?;

                if !hypothesis.supporting_evidence.is_empty() {
                    writeln!(f, "   Supporting evidence:")?;

                    for evidence in &hypothesis.supporting_evidence {
                        writeln!(f, "   {evidence}")?;
                    }
                }

                if !hypothesis.contradicting_evidence.is_empty() {
                    writeln!(f, "   Contradicting evidence:")?;

                    for evidence in &hypothesis.contradicting_evidence {
                        writeln!(f, "   {evidence}")?;
                    }
                }

                if i + 1 < self.hypotheses.len() {
                    writeln!(f)?;
                }
            }
        }

        if !self.issues.is_empty() {
            writeln!(f)?;
            writeln!(f, "Other Issues")?;
            writeln!(f, "------------")?;

            for issue in &self.issues {
                writeln!(f, "{issue}")?;
            }
        }

        if !self.evidence_gaps.is_empty() {
            writeln!(f)?;
            writeln!(f, "Evidence Gaps")?;
            writeln!(f, "------------")?;

            for gap in &self.evidence_gaps {
                writeln!(f, "- {gap}")?;
            }
        }

        Ok(())
    }
}

impl Display for Evidence {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "  - [{}] {}", self.strength, self.claim,)?;

        write!(f, " ({})", self.source)?;

        if let Some(timestamp) = self.timestamp {
            write!(f, " @ {:.3}s", timestamp)?;
        }

        writeln!(f)
    }
}

impl Display for EvidenceSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;

        if let Some(field) = &self.field {
            write!(f, ".{field}")?;
        }

        if let Some(timestamp) = self.timestamp {
            write!(f, " @ {:.3}s", timestamp)?;
        }

        if let Some(value) = self.value {
            write!(f, " = {value}")?;
        }

        Ok(())
    }
}

impl Display for EvidenceStrength {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Strong => "strong",
            Self::Moderate => "moderate",
            Self::Weak => "weak",
        };

        f.write_str(value)
    }
}

impl Display for Issue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "- {} [{}]", self.issue, self.importance,)?;

        writeln!(f, "  {}", self.description)?;

        if let Some(relevance) = &self.causal_relevance {
            writeln!(f, "  Causal relevance: {}", relevance)?;
        }

        if !self.evidence.is_empty() {
            writeln!(f, "  Evidence:")?;

            for evidence in &self.evidence {
                write!(f, "  {evidence}")?;
            }
        }

        if let Some(investigation) = &self.recommended_investigation {
            writeln!(f, "  Recommended investigation: {}", investigation)?;
        }

        Ok(())
    }
}

impl Display for IssueImportance {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Critical => "critical",
            Self::Important => "important",
            Self::Minor => "minor",
        };

        f.write_str(value)
    }
}

impl Display for CausalRelevance {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::LikelyCause => "likely cause",
            Self::PossibleCause => "possible cause",
            Self::ContributingFactor => "contributing factor",
            Self::Unrelated => "unrelated",
            Self::Unknown => "unknown",
        };

        f.write_str(value)
    }
}
