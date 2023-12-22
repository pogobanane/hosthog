use crate::ClaimCommand;
use crate::diskstate::{DiskState, Claim};
use crate::parse_timeout;
use crate::users;

pub fn do_claim(claim: &ClaimCommand, state: &mut DiskState) {
    // filter claims for exclusive claims by other users
    let other_exclusive_claims = state.claims.iter().any(|claim| claim.exclusive && claim.user != users::my_username());
    if other_exclusive_claims {
        panic!("Exclusive claim already exists. Release first.");
    }

    let timeout = parse_timeout(&claim.timeout);
    let soft_timeout = match &claim.soft_timeout {
        Some(soft_timeout) => Some(parse_timeout(soft_timeout)),
        None => None,
    }; 
    let claim = Claim {
        timeout,
        soft_timeout,
        exclusive: claim.exclusive,
        user: users::my_username(),
        comment: claim.comment.join(" "),
    };

    state.claims.push(claim.clone());
    
    println!("claim {:?}", claim);
}
