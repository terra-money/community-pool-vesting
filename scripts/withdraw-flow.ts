import * as dotenv from 'dotenv'
import { MnemonicKey, LCDClient, MsgExecuteContract } from '@terra-money/feather.js';
import * as fs from 'fs';

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

    // Get owner mnemonic, whitelisted mnemonic create the wallet and retreive the contract address
    const ownerMnemonic = new MnemonicKey({ mnemonic: process.env.MNEMONIC, index: 1 })
    const whitelistedMnemonic = new MnemonicKey({ mnemonic: process.env.MNEMONIC, index: 2 })
    const ownerWallet = lcd.wallet(ownerMnemonic);
    const contractAddres = fs.readFileSync('./scripts/address.txt').toString();

    // Execute a simple whitelist address
    let tx = await ownerWallet.createAndSignTx({
        msgs: [new MsgExecuteContract(
            ownerMnemonic.accAddress("terra"),
            contractAddres,
            {
                "add_to_whitelist": {
                    "addresses": [whitelistedMnemonic.accAddress("terra")]
                }
            },
        )],
        chainID: "pisco-1",
    });
    let broadcastRes = await lcd.tx.broadcastSync(tx, "pisco-1");
    console.log(`Whitelist address broadcasted successfully with hash: ${broadcastRes.txhash}`)
    await new Promise(resolve => setTimeout(resolve, 10000));

    // Execute a simple delegation thought the smart contract and await 30 seconds
    const whitelistedWallet = lcd.wallet(whitelistedMnemonic);
    tx = await whitelistedWallet.createAndSignTx({
        msgs: [new MsgExecuteContract(
            whitelistedMnemonic.accAddress("terra"),
            contractAddres,
            {
                "withdraw_cliff_vested_funds": {
                    "denom": "uluna",
                }
            },
        )],
        chainID: "pisco-1",
    });
    broadcastRes = await lcd.tx.broadcastSync(tx, "pisco-1");
    console.log(`Withdraw Cliff Vested Funds broadcasted successfully with hash: ${broadcastRes.txhash}`)
    await new Promise(resolve => setTimeout(resolve, 10000));

    // Execute a simple delegation thought the smart contract and await 30 seconds
    tx = await whitelistedWallet.createAndSignTx({
        msgs: [new MsgExecuteContract(
            whitelistedMnemonic.accAddress("terra"),
            contractAddres,
            {
                "withdraw_vested_funds": {
                    "denom": "uluna",
                }
            },
        )],
        chainID: "pisco-1",
    });
    broadcastRes = await lcd.tx.broadcastSync(tx, "pisco-1");
    console.log(`Withdraw Vested Funds broadcasted successfully with hash: ${broadcastRes.txhash}`)

}

init().catch(e => console.log(e))