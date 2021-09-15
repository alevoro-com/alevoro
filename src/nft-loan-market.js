import * as nearAPI from "near-api-js";
import "regenerator-runtime/runtime";
import getConfig from "./config";

const nearConfig = getConfig(process.env.NODE_ENV || "development");

const GAS = "200000000000000";

export const {
    utils: {
        format: {
            formatNearAmount, parseNearAmount
        }
    }
} = nearAPI;

let all_nfts = {};
async function connect(nearConfig) {
    // Connects to NEAR and provides `near`, `walletAccount` and `contract` objects in `window` scope
    // Initializing connection to the NEAR node.
    window.near = await nearAPI.connect({
        deps: {
            keyStore: new nearAPI.keyStores.BrowserLocalStorageKeyStore()
        },
        ...nearConfig
    });

    // Needed to access wallet login
    window.walletConnection = new nearAPI.WalletConnection(window.near);

    // Initializing our contract APIs by contract name and configuration.
    window.contract = await new nearAPI.Contract(window.walletConnection.account(), nearConfig.contractName, {
        // View methods are read-only – they don't modify the state, but usually return some value
        viewMethods: ['get_num', 'nft_tokens_for_owner', 'get_locked_tokens'],
        // Change methods can modify the state, but you don't receive the returned value when called
        changeMethods: ['increment', 'nft_mint', 'transfer_nft_to_contract', 'transfer_nft_back'],
        // Sender is the account ID to initialize transactions.
        // getAccountId() will return empty string if user is still unauthorized
        sender: window.walletConnection.getAccountId()
    });

}

function updateUI() {
    console.log("update UI");
    if (!window.walletConnection.getAccountId()) {
        document.querySelector('.score').innerHTML = "NO";
        document.querySelector('.login').innerHTML = "sign in";
    } else {
        document.querySelector('.login').innerHTML = "sign out";
        contract.get_num({account_id: window.walletConnection.getAccountId()}).then(count => {
            console.log(`get num ${count}`);
            document.querySelector('.score').innerHTML = count;
        });
    }
}

window.nearInitPromise = connect(nearConfig).then(updateUI);