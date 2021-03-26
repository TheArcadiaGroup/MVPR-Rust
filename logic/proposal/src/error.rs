#[derive(PartialEq, Debug)]
pub struct StartNotBeforeEnd;

#[derive(PartialEq, Debug)]
pub enum VotingError {
    NotEnoughVotingPower,
    ProjectDoesNotExists,
    NotAParticipant,
    VotingNotStarted,
    VotingEnded,
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
