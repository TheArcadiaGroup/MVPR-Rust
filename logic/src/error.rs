#[derive(PartialEq, Debug)]
pub struct StartNotBeforeEnd;

#[derive(PartialEq, Debug)]
pub enum VotingEngineError {
    NotEnoughVotingPower,
    NotAParticipant,
    VotingNotStarted,
    VotingEnded,
    VotingOngoing,
    VotingNotOngoing,
    ReputationAlreadyClaimed,
    NoReputationToClaim,
    VoteIsNotApproved,
    VoteDidNotFail,
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
