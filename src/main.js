import "regenerator-runtime/runtime";
import * as nearAPI from "near-api-js";
import getConfig from "./config";
import {getNFTs, viewAccountNFT} from "./nft-view/nft-view";


const nearConfig = getConfig(process.env.NODE_ENV || "development");

const GAS = "100000000000000";

export const {
    utils: {
        format: {
            formatNearAmount, parseNearAmount
        }
    }
} = nearAPI;


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
        viewMethods: [],
        // Change methods can modify the state, but you don't receive the returned value when called
        changeMethods: [],
        // Sender is the account ID to initialize transactions.
        // getAccountId() will return empty string if user is still unauthorized
        sender: window.walletConnection.getAccountId()
    });

}


function updateUI() {
    console.log("update UI");

    if (!window.walletConnection.getAccountId()) {
        document.querySelector('.login').innerHTML = "Sign In";
        document.querySelector('.account-name').innerHTML = "Please sign in";
    } else {
        document.querySelector('.login').innerHTML = "Sign out";
        document.querySelector('.account-name').innerHTML = window.walletConnection.getAccountId();

        getAccountNFTs();

    }
}


async function getAccountNFTs() {
    const wallet = window.walletConnection.getAccountId();
    const contracts = await getNFTs(wallet);

    let ptr = 0;
    const result = {index: {}, data: []};
    for (const contactId of contracts) {
        const list = await viewAccountNFT(contactId, wallet);
        if (!list || list.error || !list.length) continue;
        if (!result.index[contactId]) result.index[contactId] = {start: ptr, length: 0};
        for (let i = 0; i < list.length; i++) {
            if (!list[i] || list[i].error) continue;
            result.index[contactId].length++;
            ptr++;
            result.data.push(list[i])
        }
    }
    console.log("NFTS:");
    console.log(result);
}


document.querySelector('.login').addEventListener("click", function () {
    if (!window.walletConnection.getAccountId()) {
        walletConnection.requestSignIn(nearConfig.contractName, 'Alevoro contract');
    } else {
        walletConnection.signOut();
    }
    updateUI()
});


window.nearInitPromise = connect(nearConfig).then(updateUI);

