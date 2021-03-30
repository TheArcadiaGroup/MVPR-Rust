#[derive(PartialEq, Debug)]
pub struct StartNotBeforeEnd;

#[derive(PartialEq, Debug)]
pub enum VotingEngineError {
    VotingNotStarted,
    VotingEnded,
    VotingOngoing,
    VotingNotOngoing,
    ReputationAlreadyClaimed,
    NoReputationToClaim,
    VoteIsNotApproved,
    VoteDidNotFail,
    VoteFailed,
    InvalidReputationToStake,
    StakingLimitReached,
    AlreadyVoted,
}
#[derive(PartialEq, Debug)]
pub enum ProposalError {
    InvalidPolicingRatio,
    InvalidCategory,
    StakedRepGreaterThanReputationBalance,
    ProjectCostNotEqualToMilestonesSum,
    InvalidMilestonesProgressPercentages,
}

#[derive(PartialEq, Debug)]
pub struct NewVotingPowerBelowUsed;
