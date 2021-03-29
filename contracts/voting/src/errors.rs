use logic::VotingEngineError;
use types::ApiError;

#[repr(u16)]
pub enum Error {
    UnknownApiCommand = 1,             // 65537
    PermissionDenied = 2,              // 65538
    ThresholdViolation = 3,            // 65539
    MaxKeysLimit = 4,                  // 65540
    DuplicateKey = 5,                  // 65541
    KeyManagementThresholdError = 6,   // 65542
    DeploymentThresholdError = 7,      // 65543
    InsufficientTotalWeight = 8,       // 65544
    MissingArgument0 = 20,             // 65556
    MissingArgument1 = 21,             // 65557
    MissingArgument2 = 22,             // 65558
    InvalidArgument0 = 23,             // 65559
    InvalidArgument1 = 24,             // 65560
    InvalidArgument2 = 25,             // 65561
    UnsupportedNumberOfArguments = 30, // 65566
    NotTheAdminAccount,
    ReputationTransferFail,
    VoteDoesNotExist,
    VotingNotStarted,
    VotingEnded,
    VotingOngoing,
    VotingNotOngoing,
    ReputationAlreadyClaimed,
    NoReputationToClaim,
    VoteIsNotApproved,
    VoteDidNotFail,
    VoteFailed,
    NotAMember,
    InvalidReputationToStake,
    StakingLimitReached,
}

impl Error {
    pub fn missing_argument(i: u32) -> Error {
        match i {
            0 => Error::MissingArgument0,
            1 => Error::MissingArgument1,
            2 => Error::MissingArgument2,
            _ => Error::UnsupportedNumberOfArguments,
        }
    }

    pub fn invalid_argument(i: u32) -> Error {
        match i {
            0 => Error::InvalidArgument0,
            1 => Error::InvalidArgument1,
            2 => Error::InvalidArgument2,
            _ => Error::UnsupportedNumberOfArguments,
        }
    }
}

impl From<Error> for ApiError {
    fn from(error: Error) -> ApiError {
        ApiError::User(error as u16)
    }
}

impl From<VotingEngineError> for Error {
    fn from(error: VotingEngineError) -> Error {
        match error {
            VotingEngineError::VotingNotStarted => Error::VotingNotStarted,
            VotingEngineError::VotingEnded => Error::VotingEnded,
            VotingEngineError::VotingOngoing => Error::VotingOngoing,
            VotingEngineError::ReputationAlreadyClaimed => Error::ReputationAlreadyClaimed,
            VotingEngineError::NoReputationToClaim => Error::NoReputationToClaim,
            VotingEngineError::VoteIsNotApproved => Error::VoteIsNotApproved,
            VotingEngineError::VoteDidNotFail => Error::VoteDidNotFail,
            VotingEngineError::VoteFailed => Error::VoteFailed,
            VotingEngineError::VotingNotOngoing => Error::VoteFailed,
            VotingEngineError::InvalidReputationToStake => Error::InvalidReputationToStake,
            VotingEngineError::StakingLimitReached => Error::StakingLimitReached,
        }
    }
}
