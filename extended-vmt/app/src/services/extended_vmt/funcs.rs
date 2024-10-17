use super::utils::{Error, Result, TokenId, TokenMetadata};
use crate::services::extended_vmt::{Event, ExtendedStorage};
use sails_rs::{
    collections::{HashMap, HashSet},
    prelude::*,
};

pub fn mint(
    balances: &mut HashMap<TokenId, HashMap<ActorId, U256>>,
    total_supply: &mut HashMap<TokenId, U256>,
    storage: &mut ExtendedStorage,
    to: ActorId,
    ids: Vec<TokenId>,
    amounts: Vec<U256>,
    meta: Vec<Option<TokenMetadata>>,
) -> Result<Event> {
    if to == ActorId::zero() {
        return Err(Error::ZeroAddress);
    }

    if ids.len() != amounts.len() || ids.len() != meta.len() {
        return Err(Error::LengthMismatch);
    }

    let unique_ids: HashSet<_> = ids.clone().into_iter().collect();

    if ids.len() != unique_ids.len() {
        return Err(Error::IdIsNotUnique);
    }

    ids.iter().enumerate().try_for_each(|(i, id)| {
        if storage.token_metadata.contains_key(id) {
            return Err(Error::TokenAlreadyExists);
        } else if let Some(_token_meta) = &meta[i] {
            if amounts[i] > U256::one() {
                return Err(Error::MintMetadataToFungibleToken);
            }
        }
        Ok(())
    })?;

    for (i, meta_item) in meta.into_iter().enumerate() {
        mint_impl(storage, balances, &to, &ids[i], amounts[i], meta_item)?;
    }
    for (id, amount) in ids.iter().zip(amounts.iter()) {
        total_supply
            .entry(*id)
            .and_modify(|quantity| {
                *quantity = quantity.saturating_add(*amount);
            })
            .or_insert(*amount);
    }

    Ok(Event::Minted { to, ids, amounts })
}

fn mint_impl(
    storage: &mut ExtendedStorage,
    balances: &mut HashMap<TokenId, HashMap<ActorId, U256>>,
    account: &ActorId,
    id: &TokenId,
    amount: U256,
    meta: Option<TokenMetadata>,
) -> Result<()> {
    if let Some(metadata) = meta {
        storage.token_metadata.insert(*id, metadata);
        // since we have metadata = means we have an nft, so add it to the owners
        storage.owners.insert(*id, *account);
    }

    balances
        .entry(*id)
        .or_default()
        .entry(*account)
        .and_modify(|balance| *balance = balance.saturating_add(amount))
        .or_insert(amount);

    Ok(())
}

pub fn burn(
    balances: &mut HashMap<TokenId, HashMap<ActorId, U256>>,
    total_supply: &mut HashMap<TokenId, U256>,
    storage: &mut ExtendedStorage,
    from: ActorId,
    ids: Vec<TokenId>,
    amounts: Vec<U256>,
) -> Result<Event> {
    if ids.len() != amounts.len() {
        return Err(Error::LengthMismatch);
    }

    ids.iter()
        .zip(amounts.clone())
        .try_for_each(|(id, amount)| {
            if storage.token_metadata.contains_key(id) && amount > U256::one() {
                return Err(Error::AmountGreaterThanOneForNft);
            }
            check_opportunity_burn(balances, &from, id, amount)
        })?;

    ids.iter()
        .enumerate()
        .for_each(|(i, id)| burn_impl(storage, balances, &from, id, amounts[i]));

    for (id, amount) in ids.iter().zip(amounts.iter()) {
        let quantity = total_supply.get_mut(id).ok_or(Error::WrongId)?;
        *quantity = quantity.saturating_sub(*amount);
    }

    Ok(Event::Burned { from, ids, amounts })
}

fn check_opportunity_burn(
    balances: &mut HashMap<TokenId, HashMap<ActorId, U256>>,
    owner: &ActorId,
    id: &TokenId,
    amount: U256,
) -> Result<(), Error> {
    let zero = U256::zero();
    let balance = *balances.get(id).and_then(|m| m.get(owner)).unwrap_or(&zero);
    if balance < amount {
        return Err(Error::NotEnoughBalance);
    }
    Ok(())
}

fn burn_impl(
    storage: &mut ExtendedStorage,
    balances: &mut HashMap<TokenId, HashMap<ActorId, U256>>,
    from: &ActorId,
    id: &TokenId,
    amount: U256,
) {
    storage.owners.remove(id);
    balances
        .entry(*id)
        .or_default()
        .entry(*from)
        .and_modify(|balance| *balance = balance.saturating_sub(amount));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::extended_vmt::{ExtendedStorage, TokenMetadata};
    use sails_rs::collections::HashMap;

    fn actor_id() -> ActorId {
        1u64.into()
    }

    fn token_id() -> TokenId {
        1u64.into()
    }
    fn token_metadata() -> TokenMetadata {
        TokenMetadata {
            title: None,
            description: None,
            media: None,
            reference: None,
        }
    }

    #[test]
    fn test_mint_success() {
        let mut balances = HashMap::new();
        let mut total_supply = HashMap::new();
        let mut storage = ExtendedStorage {
            token_metadata: HashMap::new(),
            owners: HashMap::new(),
            ..Default::default()
        };

        let to = actor_id();
        let ids = vec![token_id()];
        let amounts = vec![U256::one()];
        let meta = vec![Some(token_metadata())];

        let result = mint(
            &mut balances,
            &mut total_supply,
            &mut storage,
            to,
            ids.clone(),
            amounts.clone(),
            meta,
        );

        assert!(result.is_ok());
        assert_eq!(
            balances.get(&ids[0]).unwrap().get(&to).unwrap(),
            &U256::one()
        );
        assert_eq!(total_supply.get(&ids[0]).unwrap(), &U256::one());
    }

    #[test]
    fn test_mint_zero_address() {
        let mut balances = HashMap::new();
        let mut total_supply = HashMap::new();
        let mut storage = ExtendedStorage::default();

        let to = ActorId::zero();
        let ids = vec![token_id()];
        let amounts = vec![U256::one()];
        let meta = vec![None];

        let result = mint(
            &mut balances,
            &mut total_supply,
            &mut storage,
            to,
            ids.clone(),
            amounts.clone(),
            meta,
        );

        assert_eq!(result, Err(Error::ZeroAddress));
    }

    #[test]
    fn test_mint_length_mismatch() {
        let mut balances = HashMap::new();
        let mut total_supply = HashMap::new();
        let mut storage = ExtendedStorage::default();

        let to = actor_id();
        let ids = vec![token_id()];
        let amounts = vec![U256::one(), U256::one()]; // mismatch length
        let meta = vec![None];

        let result = mint(
            &mut balances,
            &mut total_supply,
            &mut storage,
            to,
            ids.clone(),
            amounts.clone(),
            meta,
        );

        assert_eq!(result, Err(Error::LengthMismatch));
    }

    #[test]
    fn test_mint_duplicate_token_ids() {
        let mut balances = HashMap::new();
        let mut total_supply = HashMap::new();
        let mut storage = ExtendedStorage::default();

        let to = actor_id();
        let ids = vec![token_id(), token_id()]; // duplicate ids
        let amounts = vec![U256::one(), U256::one()];
        let meta = vec![None, None];

        let result = mint(
            &mut balances,
            &mut total_supply,
            &mut storage,
            to,
            ids.clone(),
            amounts.clone(),
            meta,
        );

        assert_eq!(result, Err(Error::IdIsNotUnique));
    }

    #[test]
    fn test_burn_success() {
        let mut balances: HashMap<U256, HashMap<ActorId, U256>> = HashMap::new();
        let mut total_supply = HashMap::new();
        let mut storage = ExtendedStorage {
            token_metadata: HashMap::new(),
            owners: HashMap::new(),
            ..Default::default()
        };

        let from = actor_id();
        let ids = vec![token_id()];
        let amounts = vec![U256::one()];

        // Simulate minting
        balances
            .entry(token_id())
            .or_default()
            .insert(from, U256::one());
        total_supply.insert(token_id(), U256::one());

        let result = burn(
            &mut balances,
            &mut total_supply,
            &mut storage,
            from,
            ids.clone(),
            amounts.clone(),
        );

        assert!(result.is_ok());
        assert_eq!(
            balances.get(&ids[0]).unwrap().get(&from).unwrap(),
            &U256::zero()
        );
        assert_eq!(total_supply.get(&ids[0]).unwrap(), &U256::zero());
    }

    #[test]
    fn test_burn_insufficient_balance() {
        let mut balances: HashMap<U256, HashMap<ActorId, U256>> = HashMap::new();
        let mut total_supply = HashMap::new();
        let mut storage = ExtendedStorage::default();

        let from = actor_id();
        let ids = vec![token_id()];
        let amounts: Vec<U256> = vec![5.into()];

        // Simulate a balance less than the amount to burn
        balances
            .entry(token_id())
            .or_default()
            .insert(from, U256::one());
        total_supply.insert(token_id(), U256::one());

        let result = burn(
            &mut balances,
            &mut total_supply,
            &mut storage,
            from,
            ids.clone(),
            amounts.clone(),
        );

        assert_eq!(result, Err(Error::NotEnoughBalance));
    }

    #[test]
    fn test_burn_length_mismatch() {
        let mut balances = HashMap::new();
        let mut total_supply = HashMap::new();
        let mut storage = ExtendedStorage::default();

        let from = actor_id();
        let ids = vec![token_id()];
        let amounts = vec![U256::one(), U256::one()]; // mismatch length

        let result = burn(
            &mut balances,
            &mut total_supply,
            &mut storage,
            from,
            ids.clone(),
            amounts.clone(),
        );

        assert_eq!(result, Err(Error::LengthMismatch));
    }
}
