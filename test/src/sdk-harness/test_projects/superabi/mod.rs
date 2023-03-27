use fuels::prelude::*;

abigen!(Contract(
    name = "SuperAbiTestContract",
    abi = "test_projects/superabi/out/debug/superabi-abi.json"
));

async fn get_superabi_instance() -> (SuperAbiTestContract, ContractId) {
    let wallet = launch_provider_and_get_wallet().await;
    let id = Contract::deploy(
        "test_projects/superabi/out/debug/superabi.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "test_projects/superabi/out/debug/superabi-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();
    let instance = SuperAbiTestContract::new(id.clone(), wallet);

    (instance, id.into())
}

#[tokio::test]
async fn abi_test() -> Result<()> {
    let (instance, _id) = get_superabi_instance().await;
    let contract_methods = instance.methods();

    let response = contract_methods.abi_test().call().await?;
    assert_eq!(42, response.value);

    Ok(())
}

#[tokio::test]
async fn superabi_test() -> Result<()> {
    let (instance, _id) = get_superabi_instance().await;
    let contract_methods = instance.methods();

    let response = contract_methods.superabi_test().call().await?;
    assert_eq!(41, response.value);

    Ok(())
}
