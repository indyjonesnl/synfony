use std::any::Any;
use std::sync::Arc;

use crate::token::SecurityToken;

/// Vote result from a voter.
///
/// Matches Symfony's VoterInterface constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vote {
    /// Grant access
    Granted,
    /// Deny access
    Denied,
    /// This voter has no opinion (abstain)
    Abstain,
}

/// The Voter trait — equivalent to Symfony's `VoterInterface`.
///
/// Voters make authorization decisions for specific attributes and subjects.
/// The AccessDecisionManager polls all voters and decides based on the strategy.
///
/// # Example
/// ```ignore
/// struct PostVoter;
///
/// impl Voter for PostVoter {
///     fn supports(&self, attribute: &str, subject: &dyn Any) -> bool {
///         ["EDIT", "DELETE"].contains(&attribute) && subject.is::<Post>()
///     }
///
///     fn vote(&self, token: &SecurityToken, attribute: &str, subject: &dyn Any) -> Vote {
///         let post = subject.downcast_ref::<Post>().unwrap();
///         match attribute {
///             "EDIT" if token.user_id() == post.author_id => Vote::Granted,
///             "DELETE" if token.has_role("ROLE_ADMIN") => Vote::Granted,
///             _ => Vote::Denied,
///         }
///     }
/// }
/// ```
pub trait Voter: Send + Sync {
    /// Does this voter support the given attribute + subject combination?
    fn supports(&self, attribute: &str, subject: &dyn Any) -> bool;

    /// Cast a vote for the given token, attribute, and subject.
    fn vote(&self, token: &SecurityToken, attribute: &str, subject: &dyn Any) -> Vote;
}

/// Built-in voter that checks role-based access.
///
/// Supports attributes like "ROLE_ADMIN", "ROLE_USER", etc.
/// Grants access if the token has the required role.
pub struct RoleVoter;

impl Voter for RoleVoter {
    fn supports(&self, attribute: &str, _subject: &dyn Any) -> bool {
        attribute.starts_with("ROLE_")
    }

    fn vote(&self, token: &SecurityToken, attribute: &str, _subject: &dyn Any) -> Vote {
        if token.has_role(attribute) {
            Vote::Granted
        } else {
            Vote::Denied
        }
    }
}

/// Strategy for making access decisions from multiple voter results.
#[derive(Debug, Clone, Copy)]
pub enum DecisionStrategy {
    /// Access granted if ANY voter grants (Symfony default: affirmative)
    Affirmative,
    /// Access granted only if ALL voters grant (or abstain)
    Unanimous,
    /// Access granted if more voters grant than deny
    Consensus,
}

/// The AccessDecisionManager — polls voters and decides access.
///
/// Equivalent to Symfony's `AccessDecisionManager`.
pub struct AccessDecisionManager {
    voters: Vec<Arc<dyn Voter>>,
    strategy: DecisionStrategy,
}

impl AccessDecisionManager {
    pub fn new(strategy: DecisionStrategy) -> Self {
        AccessDecisionManager {
            voters: vec![Arc::new(RoleVoter)], // RoleVoter is always registered
            strategy,
        }
    }

    /// Register a voter.
    pub fn add_voter(&mut self, voter: Arc<dyn Voter>) {
        self.voters.push(voter);
    }

    /// Check if the given token is granted the attribute for the subject.
    ///
    /// This is the equivalent of Symfony's `isGranted()`.
    pub fn is_granted(
        &self,
        token: &SecurityToken,
        attribute: &str,
        subject: &dyn Any,
    ) -> bool {
        let mut grant_count = 0;
        let mut deny_count = 0;

        for voter in &self.voters {
            if !voter.supports(attribute, subject) {
                continue;
            }

            match voter.vote(token, attribute, subject) {
                Vote::Granted => {
                    grant_count += 1;
                    if matches!(self.strategy, DecisionStrategy::Affirmative) {
                        return true;
                    }
                }
                Vote::Denied => {
                    deny_count += 1;
                    if matches!(self.strategy, DecisionStrategy::Unanimous) {
                        return false;
                    }
                }
                Vote::Abstain => {}
            }
        }

        match self.strategy {
            DecisionStrategy::Affirmative => grant_count > 0,
            DecisionStrategy::Unanimous => deny_count == 0 && grant_count > 0,
            DecisionStrategy::Consensus => grant_count > deny_count,
        }
    }
}

impl Default for AccessDecisionManager {
    fn default() -> Self {
        Self::new(DecisionStrategy::Affirmative)
    }
}
