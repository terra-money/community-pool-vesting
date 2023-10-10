import * as dotenv from 'dotenv'
import { MnemonicKey, MsgStoreCode, MsgInstantiateContract, LCDClient, Coins } from '@terra-money/feather.js';
import * as fs from 'fs';
import moment from 'moment';

dotenv.config()

const init = async () => {
    // Create the LCD Client to interact with the blockchain
    const lcd = new LCDClient({
        'pisco-1': {
            lcd: 'http://88.99.146.56:1317',
            chainID: 'pisco-1',
            gasAdjustment: 1.75,
            gasPrices: { uluna: 0.015 },
            prefix: 'terra'
        }
    })

    // Get all information from the deployer wallet
    const mk = new MnemonicKey({ mnemonic: process.env.MNEMONIC });
    const wallet = lcd.wallet(mk);
    const deployerAddress = mk.accAddress("terra");
    console.log(`Deployer address: ${deployerAddress}`)

    // Get the information for owner and recipient
    const ownerAddress = new MnemonicKey({ mnemonic: process.env.MNEMONIC, index: 1 })
        .accAddress("terra")
    const receipientAddress = new MnemonicKey({ mnemonic: process.env.MNEMONIC, index: 2 })
        .accAddress("terra")

    // Create the message and broadcast the transaction on chain
    const msgStoreCode = new MsgStoreCode(
        deployerAddress,
        fs.readFileSync('./artifacts/community_pool_vesting.wasm').toString('base64')
    );
    let tx = await wallet.createAndSignTx({
        msgs: [msgStoreCode],
        memo: "Community Pool Vesting Smart Contract",
        chainID: "pisco-1",
    });
    let result = await lcd.tx.broadcastSync(tx, "pisco-1");
    await new Promise(resolve => setTimeout(resolve, 10000));
    let txResult = await lcd.tx.txInfo(result.txhash, "pisco-1") as any;

    // Display smart contract information and wirte the id to file
    let codeId = Number(txResult.logs[0].events[1].attributes[1].value);
    console.log(`Smart contract deployed with 
    - Code ID: ${codeId}
    - Tx Hash: ${result.txhash}`);
    await new Promise(resolve => setTimeout(resolve, 3000));
    fs.writeFileSync('./scripts/code_id.txt', codeId.toString());

    // Instantiate smart contract and broadcast tx on chain
    const msgInstantiateContract = new MsgInstantiateContract(
        deployerAddress,
        deployerAddress,
        codeId,
        {
            owner: ownerAddress,
            recipient: receipientAddress,
            cliff_amount: "1",
            vesting_amount: "1000",
            start_time: moment().unix().toString(),
            end_time: moment().add(1, 'day').unix().toString(),
        },
        Coins.fromString("10000000uluna"),
        "Create a Community Pool Vesting Smart Contract"
    );
    tx = await wallet.createAndSignTx({
        msgs: [msgInstantiateContract],
        memo: "Instantiate a Community Pool Vesting Smart Contract",
        chainID: "pisco-1",
    });
    result = await lcd.tx.broadcastSync(tx, "pisco-1");
    await new Promise(resolve => setTimeout(resolve, 10000));
    txResult = await lcd.tx.txInfo(result.txhash, "pisco-1") as any;

    // Display smart contract information and write the address to file
    const address = txResult.logs[0].eventsByType.instantiate._contract_address[0];
    console.log(`Smart Contract instantiated with:
    - Code ID: ${codeId}
    - Tx Hash: ${result.txhash}
    - Contract Address: ${address}`);
    fs.writeFileSync('./scripts/address.txt', address);
}

init().catch(e => console.log(e))