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

let market_nfts = {};
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
        viewMethods: ['get_num'],
        // Change methods can modify the state, but you don't receive the returned value when called
        changeMethods: ['get_all_locked_tokens'],
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
        contract.get_all_locked_tokens({
        }).then(res => {
            const nfts = getNFTsInfo(res, true);
            showGallery(nfts);
        });

    }
}

function getNFTsInfo(res, isLocked) {
    let nfts = [];
    for (let el of res) {
        console.log(el['token_id']);
        const image_url = el['metadata']['media'];
        const title = el['metadata']['title'] || "No title";
        nfts.push(new NFT(title, el['owner_id'], el['token_id'], image_url, isLocked));
        market_nfts[el['token_id']] = new NFT(title, el['owner_id'], el['token_id'], image_url, isLocked);
    }
    return nfts
}

function showGallery(nfts){
  for (let i = 0; i < nfts.length; i++) {
    document.querySelector('.gallery').innerHTML += showNFT(nfts[i]);
  }
}

function showNFT(nft) {
    const div_info = `class=\"container_image\" id=\"${nft.token_id}\"`;
    const bottom = nft.owner === window.walletConnection.getAccountId() ? nft.owner : "Locked: " + nft.owner;
    return "<div class=\"nft\">\n" +
        "   <div class=\"nft__image\"><img " + div_info + " src=\"" + nft.url + "\" alt=\"" + nft.title + "\"></div>\n" +
        "   <h2 class=\"nft__title\">" + nft.title + "</h2>\n" +
        "   <p class=\"nft__owner\">" + bottom + "</p>\n" +
        "</div>"
}

window.nearInitPromise = connect(nearConfig).then(updateUI);