// #![no_std]
// extern crate alloc;
// use alloc::collections::BTreeMap;
// use alloc::string::String;
// use casperlabs_types::account::AccountHash;
// use core::cmp::{Eq, Ord, PartialEq, PartialOrd};

// #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// pub struct ProjectId(pub u64);

// #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
// pub struct Participant {
//     pub total_voting_power: u64,
//     pub used_voting_power: u64,
//     pub votes: BTreeMap<ProjectId, u64>,
// }

// #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
// pub struct Project {
//     pub name: String,
//     pub team_name: String,
//     pub video_link: String,
//     pub github_link: String,
//     pub google_drive_link: String,
// }

// #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
// pub struct Voting {
//     pub start_timestamp: u64,
//     pub end_timestamp: u64,
//     pub projects: BTreeMap<ProjectId, Project>,
//     pub participants: BTreeMap<AccountHash, Participant>,
// }

// type ProjectsSerialized = BTreeMap<u64, [String; 5]>;

// type ParticipantsSerialized = BTreeMap<[u8; 32], (u64, u64, BTreeMap<u64, u64>)>;

// type VotingSerialized = (
//     // (start, end)
//     (u64, u64),
//     // Projects - [name, team_name, video_ling, github_link, google_drive_link]
//     ProjectsSerialized,
//     // Participants
//     ParticipantsSerialized,
// );

// impl Voting {
//     pub fn new(start_timestamp: u64, end_timestamp: u64) -> Result<Voting, StartNotBeforeEnd> {
//         Self::validate_dates(start_timestamp, end_timestamp)?;
//         Ok(Voting {
//             participants: BTreeMap::new(),
//             projects: BTreeMap::new(),
//             start_timestamp,
//             end_timestamp,
//         })
//     }

//     pub fn start_at(&self) -> u64 {
//         self.start_timestamp
//     }

//     pub fn end_at(&self) -> u64 {
//         self.end_timestamp
//     }

//     pub fn update_dates(
//         &mut self,
//         start_timestamp: u64,
//         end_timestamp: u64,
//     ) -> Result<(), StartNotBeforeEnd> {
//         Self::validate_dates(start_timestamp, end_timestamp)?;
//         self.start_timestamp = start_timestamp;
//         self.end_timestamp = end_timestamp;
//         Ok(())
//     }

//     fn validate_dates(start_timestamp: u64, end_timestamp: u64) -> Result<(), StartNotBeforeEnd> {
//         if end_timestamp <= start_timestamp {
//             Err(StartNotBeforeEnd)
//         } else {
//             Ok(())
//         }
//     }

//     pub fn add_or_update_participant(
//         &mut self,
//         account_hash: AccountHash,
//         total_voting_power: u64,
//     ) -> Result<(), NewVotingPowerBelowUsed> {
//         match self.participants.get_mut(&account_hash) {
//             Some(participant) => {
//                 if total_voting_power < participant.used_voting_power {
//                     Err(NewVotingPowerBelowUsed)
//                 } else {
//                     participant.total_voting_power = total_voting_power;
//                     Ok(())
//                 }
//             }
//             None => {
//                 self.participants.insert(
//                     account_hash,
//                     Participant {
//                         total_voting_power,
//                         used_voting_power: 0,
//                         votes: BTreeMap::new(),
//                     },
//                 );
//                 Ok(())
//             }
//         }
//     }

//     pub fn remove_participant_if_exists(&mut self, account_hash: &AccountHash) {
//         self.participants.remove(account_hash);
//     }

//     pub fn add_or_update_project(&mut self, project_id: ProjectId, project: Project) {
//         self.projects.insert(project_id, project);
//     }

//     pub fn remove_project_if_exists_and_cancel_votes(&mut self, project_id: ProjectId) {
//         let result = self.projects.remove(&project_id);
//         if result.is_some() {
//             for (_, participant) in self.participants.iter_mut() {
//                 let vote = (*participant).votes.remove(&project_id);
//                 if let Some(value) = vote {
//                     (*participant).used_voting_power -= value;
//                 }
//             }
//         }
//     }

//     pub fn cast_vote(
//         &mut self,
//         account_hash: AccountHash,
//         project_id: ProjectId,
//         vote: u64,
//         vote_at: u64,
//     ) -> Result<(), VotingError> {
//         // Check if the vote happen in the time bounds.
//         if vote_at < self.start_timestamp {
//             return Err(VotingError::VotingNotStarted);
//         }
//         if vote_at >= self.end_timestamp {
//             return Err(VotingError::VotingEnded);
//         }

//         // Return error if the project is not on the list of all projects.
//         if !self.projects.contains_key(&project_id) {
//             return Err(VotingError::ProjectDoesNotExists);
//         }

//         // Check if the participant exists.
//         match self.participants.get_mut(&account_hash) {
//             // If the participant doesn't exists return error.
//             None => Err(VotingError::NotAParticipant),
//             // If the participant exists try to cast a vote.
//             Some(participant) => {
//                 // Read the current vote
//                 let current_vote = participant.votes.get(&project_id).unwrap_or(&0);

//                 // Check the sum of all casted votes after is not more then allowed.
//                 let new_used_voting_power = participant.used_voting_power + vote - current_vote;
//                 if new_used_voting_power > participant.total_voting_power {
//                     return Err(VotingError::NotEnoughVotingPower);
//                 }

//                 // If the vote is 0, remove project from the list.
//                 // If not, update the vote.
//                 if vote == 0 {
//                     (*participant).votes.remove(&project_id);
//                 } else {
//                     (*participant).votes.insert(project_id, vote);
//                 }

//                 // Update the used voting power.
//                 (*participant).used_voting_power = new_used_voting_power;
//                 Ok(())
//             }
//         }
//     }

//     pub fn serialize(&self) -> VotingSerialized {
//         (
//             (self.start_timestamp, self.end_timestamp),
//             self.serialize_projects(),
//             self.serialize_participants(),
//         )
//     }

//     fn serialize_projects(&self) -> ProjectsSerialized {
//         let mut output = BTreeMap::new();
//         for (key, project) in self.projects.iter() {
//             output.insert(
//                 key.0,
//                 [
//                     project.name.clone(),
//                     project.team_name.clone(),
//                     project.video_link.clone(),
//                     project.github_link.clone(),
//                     project.google_drive_link.clone(),
//                 ],
//             );
//         }
//         output
//     }

//     fn serialize_participants(&self) -> ParticipantsSerialized {
//         let mut output = BTreeMap::new();
//         for (key, participant) in self.participants.iter() {
//             let mut votes = BTreeMap::new();
//             for (project_id, vote) in participant.votes.iter() {
//                 votes.insert(project_id.0, *vote);
//             }
//             output.insert(
//                 key.value(),
//                 (
//                     participant.total_voting_power,
//                     participant.used_voting_power,
//                     votes,
//                 ),
//             );
//         }
//         output
//     }

//     pub fn deserialize(value: VotingSerialized) -> Voting {
//         Voting {
//             start_timestamp: (value.0).0,
//             end_timestamp: (value.0).1,
//             projects: Voting::deserialize_projects(value.1),
//             participants: Voting::deserialize_participants(value.2),
//         }
//     }

//     fn deserialize_projects(value: ProjectsSerialized) -> BTreeMap<ProjectId, Project> {
//         let mut output = BTreeMap::new();
//         for (id, list) in value.iter() {
//             output.insert(
//                 ProjectId(*id),
//                 Project {
//                     name: list[0].clone(),
//                     team_name: list[1].clone(),
//                     video_link: list[2].clone(),
//                     github_link: list[3].clone(),
//                     google_drive_link: list[4].clone(),
//                 },
//             );
//         }
//         output
//     }

//     fn deserialize_participants(
//         value: ParticipantsSerialized,
//     ) -> BTreeMap<AccountHash, Participant> {
//         let mut output = BTreeMap::new();
//         for (account_hash, (total_voting_power, used_voting_power, votes)) in value.iter() {
//             let mut output_votes = BTreeMap::new();
//             for (project_id, vote) in votes {
//                 output_votes.insert(ProjectId(*project_id), *vote);
//             }
//             output.insert(
//                 AccountHash::new(*account_hash),
//                 Participant {
//                     total_voting_power: *total_voting_power,
//                     used_voting_power: *used_voting_power,
//                     votes: output_votes,
//                 },
//             );
//         }
//         output
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use alloc::string::ToString;

//     pub const ALI: AccountHash = AccountHash::new([1u8; 32]);
//     pub const BOB: AccountHash = AccountHash::new([2u8; 32]);
//     pub const JON: AccountHash = AccountHash::new([3u8; 32]);

//     fn example_project(name: &str) -> Project {
//         Project {
//             name: name.to_string(),
//             team_name: "casperlabs".to_string(),
//             video_link: "https://www.youtube.com/channel/UCjFz9Sfi4yFwocnDQTWDSqA".to_string(),
//             github_link: "https://github.com/CasperLabs/CasperLabs".to_string(),
//             google_drive_link: "http://drive.google.com".to_string(),
//         }
//     }

//     #[test]
//     fn test_serialization() {
//         let mut voting = Voting::new(1, 2).unwrap();
//         let project_id = ProjectId(1);
//         let project = example_project("project");
//         let ali_power = 10;
//         voting.add_or_update_project(project_id, project);
//         voting.add_or_update_participant(ALI, ali_power).unwrap();
//         voting.cast_vote(ALI, project_id, ali_power, 1).unwrap();

//         let serialized = voting.serialize();
//         let deserialized = Voting::deserialize(serialized);
//         assert_eq!(voting, deserialized);
//     }

//     #[test]
//     fn test_init() {
//         let start = 10;
//         assert!(Voting::new(start, start).is_err());
//         assert!(Voting::new(start, start - 1).is_err());
//         let voting = Voting::new(start, start + 1).unwrap();
//         assert_eq!(voting.start_at(), start);
//         assert_eq!(voting.end_at(), start + 1);
//     }

//     #[test]
//     fn test_add_or_update_participant() {
//         let mut voting = Voting::new(1, 2).unwrap();
//         let ali_power = 10;

//         // Add participant.
//         voting.add_or_update_participant(ALI, ali_power).unwrap();
//         assert_eq!(
//             voting.participants.get(&ALI).unwrap(),
//             &Participant {
//                 total_voting_power: ali_power,
//                 used_voting_power: 0,
//                 votes: BTreeMap::new()
//             }
//         );

//         // Update particpant.
//         let updated_ali_power = 11;
//         voting
//             .add_or_update_participant(ALI, updated_ali_power)
//             .unwrap();
//         assert_eq!(
//             voting.participants.get(&ALI).unwrap(),
//             &Participant {
//                 total_voting_power: updated_ali_power,
//                 used_voting_power: 0,
//                 votes: BTreeMap::new()
//             }
//         );
//     }

//     #[test]
//     fn test_update_participant_with_vote() {
//         let mut voting = Voting::new(1, 2).unwrap();

//         // Add participant.
//         let ali_power = 10;
//         voting.add_or_update_participant(ALI, ali_power).unwrap();

//         // Add project.
//         let project_id = ProjectId(1);
//         let project = example_project("project");
//         voting.add_or_update_project(project_id, project);

//         // Make a vote.
//         let vote = 5;
//         voting.cast_vote(ALI, project_id, vote, 1).unwrap();

//         // Update voting power to the currently used voting power.
//         voting.add_or_update_participant(ALI, vote).unwrap();

//         // Assert participant.
//         let mut votes = BTreeMap::new();
//         votes.insert(project_id, vote);
//         assert_eq!(
//             voting.participants.get(&ALI).unwrap(),
//             &Participant {
//                 total_voting_power: vote,
//                 used_voting_power: vote,
//                 votes: votes
//             }
//         );

//         // Check if it's not possible to update below the used voting power.
//         assert_eq!(
//             voting.add_or_update_participant(ALI, vote - 1).unwrap_err(),
//             NewVotingPowerBelowUsed
//         );
//     }

//     #[test]
//     fn test_remove_participant() {
//         let mut voting = Voting::new(1, 2).unwrap();
//         let ali_power = 10;
//         voting.add_or_update_participant(ALI, ali_power).unwrap();
//         voting.remove_participant_if_exists(&ALI);
//         assert!(voting.participants.get(&ALI).is_none());
//     }

//     #[test]
//     fn test_add_and_update_project() {
//         let mut voting = Voting::new(1, 2).unwrap();
//         let project_id = ProjectId(1);
//         let project = example_project("project");

//         // Add new project.
//         voting.add_or_update_project(project_id, project.clone());
//         assert_eq!(voting.projects.get(&project_id).unwrap(), &project);

//         // Update the project.
//         let updated_project = example_project("project2");
//         voting.add_or_update_project(project_id, updated_project.clone());
//         assert_eq!(voting.projects.get(&project_id).unwrap(), &updated_project);
//     }

//     #[test]
//     fn test_remove_project() {
//         let mut voting = Voting::new(1, 2).unwrap();
//         let project_id = ProjectId(1);
//         let project = example_project("project");
//         voting.add_or_update_project(project_id, project);
//         voting.remove_project_if_exists_and_cancel_votes(project_id);
//         assert!(voting.projects.get(&project_id).is_none());
//     }

//     #[test]
//     fn test_voting() {
//         let mut voting = Voting::new(1, 2).unwrap();

//         // Setup projects.
//         let a_project_id = ProjectId(1);
//         let a_project = example_project("a_project");
//         let b_project_id = ProjectId(2);
//         let b_project = example_project("b_project");
//         voting.add_or_update_project(a_project_id, a_project);
//         voting.add_or_update_project(b_project_id, b_project);

//         // Setup participants.
//         let ali_power = 3;
//         let bob_power = 5;
//         voting.add_or_update_participant(ALI, ali_power).unwrap();
//         voting.add_or_update_participant(BOB, bob_power).unwrap();

//         // Cannot vote before voting starts.
//         let vote_at = 0;
//         assert_eq!(
//             voting.cast_vote(ALI, a_project_id, 1, vote_at).unwrap_err(),
//             VotingError::VotingNotStarted
//         );

//         // Cannot vote after voting ends.
//         let vote_at = 2;
//         assert_eq!(
//             voting.cast_vote(ALI, a_project_id, 1, vote_at).unwrap_err(),
//             VotingError::VotingEnded
//         );

//         // Cannot vote more then voting power.
//         let vote_at = 1;
//         assert_eq!(
//             voting
//                 .cast_vote(ALI, b_project_id, ali_power + 1, vote_at)
//                 .unwrap_err(),
//             VotingError::NotEnoughVotingPower
//         );

//         // Cannot vote on non exisitng projects.
//         let unknown_project_id = ProjectId(3);
//         assert_eq!(
//             voting
//                 .cast_vote(ALI, unknown_project_id, ali_power, vote_at)
//                 .unwrap_err(),
//             VotingError::ProjectDoesNotExists
//         );

//         // Cannot vote as non participant.
//         assert_eq!(
//             voting
//                 .cast_vote(JON, a_project_id, ali_power, vote_at)
//                 .unwrap_err(),
//             VotingError::NotAParticipant
//         );

//         // Vote as Ali.
//         let ali_a_project_vote = 2;
//         let ali_b_project_vote = 1;
//         voting
//             .cast_vote(ALI, a_project_id, ali_a_project_vote, vote_at)
//             .unwrap();
//         voting
//             .cast_vote(ALI, b_project_id, ali_b_project_vote, vote_at)
//             .unwrap();
//         let mut votes = BTreeMap::new();
//         votes.insert(a_project_id, ali_a_project_vote);
//         votes.insert(b_project_id, ali_b_project_vote);
//         assert_eq!(
//             voting.participants.get(&ALI).unwrap(),
//             &Participant {
//                 total_voting_power: ali_power,
//                 used_voting_power: ali_a_project_vote + ali_b_project_vote,
//                 votes: votes
//             }
//         );

//         // Vote as Bob.
//         let bob_a_project_vote = 1;
//         let bob_b_project_vote = 2;
//         voting
//             .cast_vote(BOB, a_project_id, bob_a_project_vote, vote_at)
//             .unwrap();
//         voting
//             .cast_vote(BOB, b_project_id, bob_b_project_vote, vote_at)
//             .unwrap();
//         let mut votes = BTreeMap::new();
//         votes.insert(a_project_id, bob_a_project_vote);
//         votes.insert(b_project_id, bob_b_project_vote);
//         assert_eq!(
//             voting.participants.get(&BOB).unwrap(),
//             &Participant {
//                 total_voting_power: bob_power,
//                 used_voting_power: bob_a_project_vote + bob_b_project_vote,
//                 votes: votes
//             }
//         );

//         // Removing project should remove votes.
//         voting.remove_project_if_exists_and_cancel_votes(b_project_id);
//         let mut votes = BTreeMap::new();
//         votes.insert(a_project_id, ali_a_project_vote);
//         assert_eq!(
//             voting.participants.get(&ALI).unwrap(),
//             &Participant {
//                 total_voting_power: ali_power,
//                 used_voting_power: ali_a_project_vote,
//                 votes: votes
//             }
//         );
//         let mut votes = BTreeMap::new();
//         votes.insert(a_project_id, bob_a_project_vote);
//         assert_eq!(
//             voting.participants.get(&BOB).unwrap(),
//             &Participant {
//                 total_voting_power: bob_power,
//                 used_voting_power: bob_a_project_vote,
//                 votes: votes
//             }
//         );
//     }

//     #[test]
//     fn test_voting_update() {
//         let mut voting = Voting::new(1, 2).unwrap();

//         // Add new project.
//         let project_id = ProjectId(1);
//         let project = example_project("project");
//         voting.add_or_update_project(project_id, project.clone());
//         assert_eq!(voting.projects.get(&project_id).unwrap(), &project);

//         // Add participant.
//         let ali_power = 10;
//         voting.add_or_update_participant(ALI, ali_power).unwrap();

//         // Make the first vote.
//         let first_vote = 5;
//         voting.cast_vote(ALI, project_id, first_vote, 1).unwrap();

//         // Assert the first vote.
//         let mut votes = BTreeMap::new();
//         votes.insert(project_id, first_vote);
//         assert_eq!(
//             voting.participants.get(&ALI).unwrap(),
//             &Participant {
//                 total_voting_power: ali_power,
//                 used_voting_power: first_vote,
//                 votes: votes
//             }
//         );

//         // Make the second vote.
//         let second_vote = 3;
//         voting.cast_vote(ALI, project_id, second_vote, 1).unwrap();

//         // Assert the second vote.
//         let mut votes = BTreeMap::new();
//         votes.insert(project_id, second_vote);
//         assert_eq!(
//             voting.participants.get(&ALI).unwrap(),
//             &Participant {
//                 total_voting_power: ali_power,
//                 used_voting_power: second_vote,
//                 votes: votes
//             }
//         );

//         // Remove vote if the voting power is 0.
//         voting.cast_vote(ALI, project_id, 0, 1).unwrap();
//         assert_eq!(
//             voting.participants.get(&ALI).unwrap(),
//             &Participant {
//                 total_voting_power: ali_power,
//                 used_voting_power: 0,
//                 votes: BTreeMap::new()
//             }
//         );
//     }

//     #[test]
//     fn test_update_dates() {
//         let mut voting = Voting::new(1, 2).unwrap();
//         let new_start = 5;
//         let new_end = 10;

//         // Test errors.
//         let result = voting.update_dates(new_end, new_start);
//         assert_eq!(result, Err(StartNotBeforeEnd));

//         // Test updates.
//         let result = voting.update_dates(new_start, new_end);
//         assert!(result.is_ok());
//         assert_eq!(voting.start_at(), new_start);
//         assert_eq!(voting.end_at(), new_end);
//     }
// }
