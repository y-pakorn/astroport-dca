use std::error::Error;

use astroport_dca::{DcaInfo, ExecuteMsg, QueryMsg, UserDcaInfo};
use cosmwasm_std::{Addr, Coin, Uint128};
use cw_multi_test::Executor;

use crate::error::ContractError;

use super::common::*;

#[test]
fn create_order_not_duplicate() -> Result<(), Box<dyn Error>> {
    let (mut app, dca) = instantiate();

    let err = app
        .execute_contract(
            Addr::unchecked(USER_ONE),
            dca,
            &ExecuteMsg::CreateDcaOrder {
                initial_asset: native_asset(USDC, 50_000_000),
                target_asset: native_info(USDC),
                interval: 600,
                dca_amount: Uint128::new(10_000_000),
                start_at: None,
                config_override: None,
            },
            &[Coin::new(50_000_000, USDC)],
        )
        .unwrap_err();

    assert_eq!(
        err.downcast::<ContractError>()?,
        ContractError::DuplicateAsset {}
    );

    Ok(())
}

#[test]
fn create_order_dca_divisible() -> Result<(), Box<dyn Error>> {
    let (mut app, dca) = instantiate();

    let err = app
        .execute_contract(
            Addr::unchecked(USER_ONE),
            dca,
            &ExecuteMsg::CreateDcaOrder {
                initial_asset: native_asset(USDC, 50_000_000),
                target_asset: native_info(USDT),
                interval: 600,
                dca_amount: Uint128::new(15_000_000),
                start_at: None,
                config_override: None,
            },
            &[Coin::new(50_000_000, USDC)],
        )
        .unwrap_err();

    assert_eq!(
        err.downcast::<ContractError>()?,
        ContractError::IndivisibleDeposit {}
    );

    Ok(())
}

#[test]
fn create_order_not_too_small() -> Result<(), Box<dyn Error>> {
    let (mut app, dca) = instantiate();

    let err = app
        .execute_contract(
            Addr::unchecked(USER_ONE),
            dca,
            &ExecuteMsg::CreateDcaOrder {
                initial_asset: native_asset(USDC, 50_000_000),
                target_asset: native_info(USDT),
                interval: 600,
                dca_amount: Uint128::new(60_000_000),
                start_at: None,
                config_override: None,
            },
            &[Coin::new(50_000_000, USDC)],
        )
        .unwrap_err();

    assert_eq!(
        err.downcast::<ContractError>()?,
        ContractError::DepositTooSmall {}
    );

    Ok(())
}

#[test]
fn create_order() -> Result<(), Box<dyn Error>> {
    let (mut app, dca) = instantiate();

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        dca.clone(),
        &ExecuteMsg::CreateDcaOrder {
            initial_asset: native_asset(USDC, 50_000_000),
            target_asset: native_info(LUNA),
            interval: 600,
            dca_amount: Uint128::new(10_000_000),
            start_at: None,
            config_override: None,
        },
        &[Coin::new(50_000_000, USDC)],
    )?;

    let infos: Vec<UserDcaInfo> = app.wrap().query_wasm_smart(
        &dca,
        &QueryMsg::UserDcaOrders {
            user: USER_ONE.to_string(),
        },
    )?;

    assert_eq!(infos.len(), 1);

    let UserDcaInfo {
        info:
            DcaInfo {
                id,
                owner,
                initial_asset,
                target_asset,
                interval,
                last_purchase,
                dca_amount,
                ..
            },
        ..
    } = infos.into_iter().next().unwrap();
    assert_eq!(id, 0);
    assert_eq!(owner, USER_ONE);
    assert_eq!(initial_asset, native_asset(USDC, 50_000_000));
    assert_eq!(target_asset, native_info(LUNA));
    assert_eq!(interval, 600);
    assert_eq!(last_purchase, 1);
    assert_eq!(dca_amount.u128(), 10_000_000);

    Ok(())
}

#[test]
fn create_multiple_orders_same_asset() -> Result<(), Box<dyn Error>> {
    let (mut app, dca) = instantiate();

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        dca.clone(),
        &ExecuteMsg::CreateDcaOrder {
            initial_asset: native_asset(USDC, 50_000_000),
            target_asset: native_info(LUNA),
            interval: 600,
            dca_amount: Uint128::new(10_000_000),
            start_at: None,
            config_override: None,
        },
        &[Coin::new(50_000_000, USDC)],
    )?;

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        dca.clone(),
        &ExecuteMsg::CreateDcaOrder {
            initial_asset: native_asset(USDC, 50_000_000),
            target_asset: native_info(USDT),
            interval: 600,
            dca_amount: Uint128::new(10_000_000),
            start_at: None,
            config_override: None,
        },
        &[Coin::new(50_000_000, USDC)],
    )?;

    let infos: Vec<UserDcaInfo> = app.wrap().query_wasm_smart(
        &dca,
        &QueryMsg::UserDcaOrders {
            user: USER_ONE.to_string(),
        },
    )?;

    assert_eq!(infos.len(), 2);

    let mut infos_iter = infos.into_iter();
    let second_order = infos_iter.next().unwrap();
    assert_eq!(second_order.info.id, 1);
    assert_eq!(
        second_order.info.initial_asset,
        native_asset(USDC, 50_000_000)
    );
    let first_order = infos_iter.next().unwrap();
    assert_eq!(first_order.info.id, 0);
    assert_eq!(
        first_order.info.initial_asset,
        native_asset(USDC, 50_000_000)
    );

    Ok(())
}

#[test]
fn create_multiple_orders_diff_asset() -> Result<(), Box<dyn Error>> {
    let (mut app, dca) = instantiate();

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        dca.clone(),
        &ExecuteMsg::CreateDcaOrder {
            initial_asset: native_asset(USDC, 50_000_000),
            target_asset: native_info(LUNA),
            interval: 600,
            dca_amount: Uint128::new(10_000_000),
            start_at: None,
            config_override: None,
        },
        &[Coin::new(50_000_000, USDC)],
    )?;

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        dca.clone(),
        &ExecuteMsg::CreateDcaOrder {
            initial_asset: native_asset(USDT, 50_000_000),
            target_asset: native_info(LUNA),
            interval: 600,
            dca_amount: Uint128::new(10_000_000),
            start_at: None,
            config_override: None,
        },
        &[Coin::new(50_000_000, USDT)],
    )?;

    let infos: Vec<UserDcaInfo> = app.wrap().query_wasm_smart(
        &dca,
        &QueryMsg::UserDcaOrders {
            user: USER_ONE.to_string(),
        },
    )?;

    assert_eq!(infos.len(), 2);

    let mut infos_iter = infos.into_iter();
    let second_order = infos_iter.next().unwrap();
    assert_eq!(second_order.info.id, 1);
    assert_eq!(
        second_order.info.initial_asset,
        native_asset(USDT, 50_000_000)
    );
    let first_order = infos_iter.next().unwrap();
    assert_eq!(first_order.info.id, 0);
    assert_eq!(
        first_order.info.initial_asset,
        native_asset(USDC, 50_000_000)
    );

    Ok(())
}

#[test]
fn create_multiple_orders_multiple_users() -> Result<(), Box<dyn Error>> {
    let (mut app, dca) = instantiate();

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        dca.clone(),
        &ExecuteMsg::CreateDcaOrder {
            initial_asset: native_asset(USDC, 50_000_000),
            target_asset: native_info(LUNA),
            interval: 600,
            dca_amount: Uint128::new(10_000_000),
            start_at: None,
            config_override: None,
        },
        &[Coin::new(50_000_000, USDC)],
    )?;

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        dca.clone(),
        &ExecuteMsg::CreateDcaOrder {
            initial_asset: native_asset(USDT, 50_000_000),
            target_asset: native_info(LUNA),
            interval: 600,
            dca_amount: Uint128::new(10_000_000),
            start_at: None,
            config_override: None,
        },
        &[Coin::new(50_000_000, USDT)],
    )?;

    app.execute_contract(
        Addr::unchecked(USER_THREE),
        dca.clone(),
        &ExecuteMsg::CreateDcaOrder {
            initial_asset: native_asset(USDT, 50_000_000),
            target_asset: native_info(LUNA),
            interval: 600,
            dca_amount: Uint128::new(10_000_000),
            start_at: None,
            config_override: None,
        },
        &[Coin::new(50_000_000, USDT)],
    )?;

    let infos: Vec<DcaInfo> = app.wrap().query_wasm_smart(
        &dca,
        &QueryMsg::AllDcaOrders {
            start_after: None,
            limit: None,
            is_ascending: None,
        },
    )?;
    assert_eq!(infos.len(), 3);
    assert_eq!(
        infos
            .into_iter()
            .map(|e| (e.id, e.owner.to_string()))
            .collect::<Vec<_>>(),
        vec![
            (2u64, USER_THREE.to_string()),
            (1u64, USER_ONE.to_string()),
            (0u64, USER_ONE.to_string())
        ],
    );

    Ok(())
}
