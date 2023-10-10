import * as dotenv from 'dotenv'
import { MnemonicKey, MsgStoreCode, MsgInstantiateContract, LCDClient, Coins, MsgExecuteContract } from '@terra-money/feather.js';
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

    // Get owner address, create the wallet and retreive the contract address
    const ownerAddress = new MnemonicKey({ mnemonic: process.env.MNEMONIC, index: 1 })
    const wallet = lcd.wallet(ownerAddress);
    const contractAddres = fs.readFileSync('./scripts/address.txt').toString();

    // Execute a simple delegation thought the smart contract and await 30 seconds
    let tx = await wallet.createAndSignTx({
        msgs: [new MsgExecuteContract(
            ownerAddress.accAddress("terra"),
            contractAddres,
            {
                "delegate_funds": {
                    "validator": "terravaloper1zdpgj8am5nqqvht927k3etljyl6a52kwqndjz2",
                    "amount": {
                        "amount": "300000",
                        "denom": "uluna"
                    },
                }
            },
        )],
        memo: "Simple delegation throught Community Pool Vesting Smart Contract",
        chainID: "pisco-1",
    });
    let broadcastRes = await lcd.tx.broadcastSync(tx, "pisco-1");
    console.log(`Delegation broadcasted successfully with hash: ${broadcastRes.txhash}`)
    await new Promise(resolve => setTimeout(resolve, 30000));


    // Execute an redelegation from the smart contract and wait another 30 seconds
    tx = await wallet.createAndSignTx({
        msgs: [new MsgExecuteContract(
            ownerAddress.accAddress("terra"),
            contractAddres,
            {
                "redelegate_funds": {
                    "src_validator": "terravaloper1zdpgj8am5nqqvht927k3etljyl6a52kwqndjz2",
                    "dst_validator": "terravaloper13sulzl3p0wk2t0x7aws7w8glmrh83z4y8atvgr",
                    "amount": {
                        "amount": "100000",
                        "denom": "uluna"
                    },
                }
            },
        )],
        memo: "Simple redelegation throught Community Pool Vesting Smart Contract",
        chainID: "pisco-1",
    });
    broadcastRes = await lcd.tx.broadcastSync(tx, "pisco-1");
    console.log(`Redelegation broadcasted successfully with hash: ${broadcastRes.txhash}`)
    await new Promise(resolve => setTimeout(resolve, 30000));

    // Execute an undelegation from the smart contract
    tx = await wallet.createAndSignTx({
        msgs: [new MsgExecuteContract(
            ownerAddress.accAddress("terra"),
            contractAddres,
            {
                "undelegate_funds": {
                    "validator": "terravaloper1zdpgj8am5nqqvht927k3etljyl6a52kwqndjz2",
                    "amount": {
                        "amount": "100000",
                        "denom": "uluna"
                    },
                }
            },
        )],
        memo: "Simple delegation throught Community Pool Vesting Smart Contract",
        chainID: "pisco-1",
    });
    broadcastRes = await lcd.tx.broadcastSync(tx, "pisco-1");
    console.log(`Undelegation broadcasted successfully with hash: ${broadcastRes.txhash}`)
}

init().catch(e => console.log(e))