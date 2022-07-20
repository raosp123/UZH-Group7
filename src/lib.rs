use concordium_std::*;

//This just lets us have these countries as constants in byte form
pub mod countries {
    pub const CH: &[u8; 2] = b"CH"; //Swiss
    pub const DK: &[u8; 2] = b"DK"; //Denmark
}

//Used to check if User attributes are partially or fully validated
#[derive(Serialize, Debug, PartialEq, Eq)]
enum Quantifier {
    Any,
    All,
}

/// This is like a class/struct that holds what the valid nationalities are
#[derive(Serialize, SchemaType)]
struct NationalityPolicy {
    //List of valid residences
    //As in this case, Swiss
    allowed_nationality: Vec<Vec<u8>>,
    //Should this hold for all or some credential
    scope: Quantifier,
}

/// This implements the checks for the user's nationality.
/// For now we are checking if the NATIONALITY matches the swiss code "CH", if it does our function returns true.
/// Ignore the other comments inside the function, that is from the example code they sent us, I leave them there for now
impl NationalityPolicy {
    //TODO: Here it would be nice to take the ctx.policies() iterator as argument instead of ctx
    fn is_satisfied<CD: HasCommonData>(&self, policies: CD::PolicyIteratorType) -> bool {
        //Iterate over all account policies
        for mut policy in policies {
            //Iterate over attribtues of an account policy
            let mut policy_ok = false;
            let mut buf: [u8; 31] = [0; 31];
            while let Some((tag, len)) = policy.next_item(&mut buf) {
                if tag == attributes::NATIONALITY {
                    let country = buf[0..len.into()].to_vec();
                    if len == 2 && self.allowed_nationality.contains(&country) {
                        //We have found one credential which satisfies the policy
                        if self.scope == Quantifier::Any {
                            return true;
                        }
                        policy_ok = true;
                    }
                }
            }
            //We found a credential that did not contain the right attribute
            if !policy_ok && self.scope == Quantifier::All {
                return false;
            }
        }
        //doesn't fit in any policy
        if self.scope == Quantifier::Any {
            return false;
        }
        return true;
    }
}

#[derive(Serialize, SchemaType)]
struct AgePolicy {
    //Date Of Birth must be in this range [minimal_DOB,maximal_DOB]
    minimal_dob: u64,
    maximal_dob: u64,
    //Should this hold for all or some credential
    scope: Quantifier,
}

/// This implements the checks for the user's age.
/// date of birth is used for checking
impl AgePolicy {
    fn is_satisfied<CD: HasCommonData>(&self, policies: CD::PolicyIteratorType) -> bool {
        //Iterate over all account policies
        for mut policy in policies {
            //Iterate over attribtues of an account policy
            let mut policy_ok: bool = false;
            let mut buf: [u8; 31] = [0; 31];
            while let Some((tag, len)) = policy.next_item(&mut buf) {
                if tag == attributes::DOB {
                    //convert into a u64 decimal date
                    let mut date = buf[0..len.into()].to_vec();
                    let mut _dob: u64 = 0;
                    let mut cnt: u64 = 0u64;
                    loop {
                        _dob = _dob * 10 + date[cnt as usize] as u64;
                        cnt += 1;
                        if cnt as usize == buf.len() {
                            break;
                        }
                    }
                    if (self.minimal_dob <= _dob) && (self.maximal_dob >= _dob) {
                        //We have found one credential which satisfies the policy
                        if self.scope == Quantifier::Any {
                            return true;
                        }
                        policy_ok = true;
                    }
                }
            }
            //We found a credential that did not contain the right attribute
            if !policy_ok && self.scope == Quantifier::All {
                return false;
            }
        }
        //doesn't fit in any policy
        if self.scope == Quantifier::Any {
            return false;
        }
        return true;
    }
}

/// This is the state of the smart contract, A.K.A the permanent variables that it holds.
/// For testing it currently holds the total number of votes, and the NationalityPolicy struct
#[derive(Serialize, SchemaType)]
struct State {
    total_votes: u64,
    nationality_policy: NationalityPolicy,
    //age_policy: AgePolicy,
}

//This is Error throwing, can ignore
//Age violation and Already Voted added(not necessarily to use)
#[derive(Reject, Debug, PartialEq, Eq)]
enum ReceiveError {
    NotAnAccount,
    NationalityPolicyViolation,
    AgePolicyViolation,
    AlreadyVoted,
}

///The initialisation of the contract, as you can see there is some initial setup
/// and then we start setting up the contract initial state. We explicitly state that only
/// Swiss (CH) Nationalities are accepted
#[init(contract = "voting")]
#[inline(always)]
fn contract_init<'a, S: HasStateApi>(
    _ctx: &impl HasInitContext,
    _state_builder: &mut StateBuilder<S>,
) -> InitResult<State> {
    // For simplicity, the nationality policy is hardcoded (instead of being read as parameter)
    let nationality_policy = NationalityPolicy {
        allowed_nationality: vec![countries::CH.to_vec()],
        scope: Quantifier::All,
    };
    let state = State {
        total_votes: 0u64,
        nationality_policy,
        //age_policy,
    };
    Ok(state)
}

///This is the function called when the contract is receiving something, i.e when a user tries to run the contract after it has been initialised
/// at the moment for simplicity and testing, when a user successfully calls the contract and they have a valid nationality, the vote counter is incremeneted by one
#[receive(contract = "voting", name = "vote-increment", mutable)]
fn just_increment<'a, S: HasStateApi, RC: HasReceiveContext>(
    ctx: &RC,
    host: &mut impl HasHost<State, StateApiType = S>,
) -> Result<u64, ReceiveError> {
    // Only allow accounts to increment the counter
    if let Address::Contract(_) = ctx.sender() {
        bail!(ReceiveError::NotAnAccount)
    }

    // Only allow accounts that satisfy the nationality policy
    ensure!(
        host.state()
            .nationality_policy
            .is_satisfied::<RC>(ctx.policies()),
        ReceiveError::NationalityPolicyViolation
    );

    //Increment total votes
    host.state_mut().total_votes += 1;
    Ok(host.state().total_votes)
}

///Unit
/// Testing
#[concordium_cfg_test]
mod tests {
    use super::*;
    use concordium_std::test_infrastructure::*;

    #[concordium_test]
    fn test_counter() {
        let account1 = AccountAddress([1u8; 32]);
        let amount = Amount::from_micro_ccd(0);

        //Create test state
        let state_builder = TestStateBuilder::new();
        let nationality_policy = NationalityPolicy {
            allowed_nationality: vec![countries::CH.to_vec()],
            scope: Quantifier::All,
        };
        let state = State {
            total_votes: 0u64,
            nationality_policy,
        };
        let mut host = TestHost::new(state, state_builder);
        host.set_self_balance(amount);

        //Test 1: Increment counter with valid attributes
        let mut ctx = TestReceiveContext::empty();
        ctx.metadata_mut()
            .set_slot_time(Timestamp::from_timestamp_millis(100));
        ctx.set_sender(Address::Account(account1));
        let attr = vec![
            (attributes::NATIONALITY, countries::CH.to_vec()),
            (attributes::COUNTRY_OF_RESIDENCE, countries::DK.to_vec()),
        ];
        let policy = Policy {
            created_at: Timestamp::from_timestamp_millis(0),
            identity_provider: 1,
            valid_to: Timestamp::from_timestamp_millis(100),
            items: attr,
        };
        ctx.push_policy(policy);

        let res = just_increment(&ctx, &mut host);
        let res = dbg!(res);
        claim!(res.is_ok(), "Should work");
    }
}
