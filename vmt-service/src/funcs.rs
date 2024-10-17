use super::utils::{Error, Result, *};
use crate::Event;
use sails_rs::{
    collections::{HashMap, HashSet},
    prelude::*,
    ActorId,
};

pub fn approve(
    allowances: &mut HashMap<ActorId, HashSet<ActorId>>,
    owner: ActorId,
    to: ActorId,
) -> Result<bool> {
    if owner == to {
        return Ok(false);
    }

    if to == ActorId::zero() {
        return Err(Error::ZeroAddress);
    }

    allowances
        .entry(owner)
        .and_modify(|approvals| {
            approvals.insert(to);
        })
        .or_insert_with(|| HashSet::from([to]));

    Ok(true)
}

pub fn transfer_from(
    balances: &mut HashMap<TokenId, HashMap<ActorId, U256>>,
    allowances: &HashMap<ActorId, HashSet<ActorId>>,
    msg_src: ActorId,
    from: ActorId,
    to: ActorId,
    ids: Vec<TokenId>,
    amounts: Vec<U256>,
) -> Result<Event, Error> {
    if from == to {
        return Err(Error::SenderAndRecipientAddressesAreSame);
    }

    if from != msg_src && !is_approved(allowances, &from, &msg_src) {
        return Err(Error::CallerIsNotOwnerOrApproved);
    }

    if to == ActorId::zero() {
        return Err(Error::ZeroAddress);
    }

    if ids.len() != amounts.len() {
        return Err(Error::LengthMismatch);
    }

    for (id, amount) in ids.iter().zip(amounts.clone()) {
        check_opportunity_transfer(&*balances, &from, id, amount)?;
    }

    for (i, id) in ids.iter().enumerate() {
        transfer_from_impl(balances, &from, &to, id, amounts[i]);
    }

    Ok(Event::Transfer {
        from,
        to,
        ids,
        amounts,
    })
}

fn transfer_from_impl(
    balances: &mut HashMap<TokenId, HashMap<ActorId, U256>>,
    from: &ActorId,
    to: &ActorId,
    id: &TokenId,
    amount: U256,
) {
    balances
        .entry(*id)
        .or_default()
        .entry(*from)
        .and_modify(|balance| *balance = balance.saturating_sub(amount));

    balances
        .entry(*id)
        .or_default()
        .entry(*to)
        .and_modify(|balance| *balance = balance.saturating_add(amount))
        .or_insert(amount);
}

fn check_opportunity_transfer(
    balances: &HashMap<TokenId, HashMap<ActorId, U256>>,
    from: &ActorId,
    id: &TokenId,
    amount: U256,
) -> Result<(), Error> {
    let balance = get_balance(balances, from, id).saturating_sub(amount);

    if balance < amount {
        return Err(Error::InsufficientBalance);
    }
    Ok(())
}

pub fn is_approved(
    allowances: &HashMap<ActorId, HashSet<ActorId>>,
    account: &ActorId,
    operator: &ActorId,
) -> bool {
    if let Some(approvals) = allowances.get(account) {
        return approvals.contains(operator);
    }
    false
}

pub fn get_balance(
    balances: &HashMap<TokenId, HashMap<ActorId, U256>>,
    account: &ActorId,
    id: &TokenId,
) -> U256 {
    let zero = U256::zero();
    *balances
        .get(id)
        .and_then(|m| m.get(account))
        .unwrap_or(&zero)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::funcs;
    use utils::*;

    #[test]
    fn approve() {
        // Initializing thread logger.
        let _ = env_logger::try_init();

        // Creating empty allowances map
        let mut allowances = HashMap::new();

        // # Test case #1: Approve another actor.
        {
            let owner = alice();
            let approved = bob();
            assert!(allowances.is_empty());
            assert_eq!(funcs::approve(&mut allowances, owner, approved), Ok(true));
            assert!(allowances.contains_key(&owner));
            assert!(allowances.get(&owner).unwrap().contains(&approved));
        }

        // # Test case #2: Error when approving self.
        {
            let owner = alice();
            assert_eq!(funcs::approve(&mut allowances, owner, owner), Ok(false));
        }

        // # Test case #3: Approve zero address (should fail).
        {
            let owner = alice();
            let approved = ActorId::zero();
            assert_eq!(
                funcs::approve(&mut allowances, owner, approved),
                Err(Error::ZeroAddress)
            );
        }
    }

    #[test]
    fn transfer_from() {
        // Initializing thread logger.
        let _ = env_logger::try_init();

        // Creating maps for balances and allowances
        let mut balances = HashMap::new();
        let mut allowances = HashMap::new();

        let token_id = 1.into();
        let owner = alice();
        let approved = bob();
        let recipient = dave();

        // Adding some balance to the owner
        balances.insert(token_id, HashMap::from([(owner, 100.into())]));

        // Approving the transfer
        funcs::approve(&mut allowances, owner, approved).unwrap();

        // # Test case #1: Successful transfer.
        assert_eq!(
            funcs::transfer_from(
                &mut balances,
                &allowances,
                approved,
                owner,
                recipient,
                vec![token_id],
                vec![50.into()]
            ),
            Ok(Event::Transfer {
                from: owner,
                to: recipient,
                ids: vec![token_id],
                amounts: vec![50.into()],
            })
        );

        // Check that the balances are updated
        assert_eq!(get_balance(&balances, &owner, &token_id), 50.into());
        assert_eq!(get_balance(&balances, &recipient, &token_id), 50.into());

        // # Test case #2: Transfer from owner to same recipient (should fail).
        assert_eq!(
            funcs::transfer_from(
                &mut balances,
                &allowances,
                owner,
                owner,
                owner,
                vec![token_id],
                vec![50.into()]
            ),
            Err(Error::SenderAndRecipientAddressesAreSame)
        );

        // # Test case #3: Transfer to zero address (should fail).
        assert_eq!(
            funcs::transfer_from(
                &mut balances,
                &allowances,
                approved,
                owner,
                ActorId::zero(),
                vec![token_id],
                vec![50.into()]
            ),
            Err(Error::ZeroAddress)
        );

        // # Test case #4: Insufficient balance.
        assert_eq!(
            funcs::transfer_from(
                &mut balances,
                &allowances,
                approved,
                owner,
                recipient,
                vec![token_id],
                vec![100.into()]
            ),
            Err(Error::InsufficientBalance)
        );
    }

    mod utils {
        use super::*;

        pub fn alice() -> ActorId {
            1u64.into()
        }

        pub fn bob() -> ActorId {
            2u64.into()
        }

        pub fn dave() -> ActorId {
            4u64.into()
        }
    }
}
