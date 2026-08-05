//! Central scientific-authorization gate for the SHD instrument.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstrumentState {
    Uncalibrated,
    Calibrated,
}

pub const SHD_INSTRUMENT_STATE: InstrumentState = InstrumentState::Uncalibrated;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampaignKind {
    Calibration,
    Parity,
    HarnessValidation,
    LocalLearning,
    Transfer,
    Optimizer,
}

impl CampaignKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Calibration => "calibration",
            Self::Parity => "parity",
            Self::HarnessValidation => "harness-validation",
            Self::LocalLearning => "local-learning",
            Self::Transfer => "transfer",
            Self::Optimizer => "optimizer",
        }
    }
}

pub fn authorize_campaign(kind: CampaignKind) -> Result<(), String> {
    match (SHD_INSTRUMENT_STATE, kind) {
        (
            InstrumentState::Uncalibrated,
            CampaignKind::Calibration | CampaignKind::Parity | CampaignKind::HarnessValidation,
        )
        | (InstrumentState::Calibrated, _) => Ok(()),
        (InstrumentState::Uncalibrated, blocked) => Err(format!(
            "SHD instrument is UNCALIBRATED; blocked {} campaign. Only calibration, parity, \
             and harness-validation work is authorized. See results/SHD_INSTRUMENT_STATUS.md",
            blocked.as_str()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uncalibrated_state_allows_only_instrument_work() {
        assert!(authorize_campaign(CampaignKind::Calibration).is_ok());
        assert!(authorize_campaign(CampaignKind::Parity).is_ok());
        assert!(authorize_campaign(CampaignKind::HarnessValidation).is_ok());
        assert!(authorize_campaign(CampaignKind::LocalLearning).is_err());
        assert!(authorize_campaign(CampaignKind::Transfer).is_err());
        assert!(authorize_campaign(CampaignKind::Optimizer).is_err());
    }
}
