import * as nearAPI from "near-api-js";
import "regenerator-runtime/runtime";
import getConfig from "./config";
import {NFT, LockedNFT} from "./classes.js"

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
        viewMethods: ['get_num', 'get_all_locked_tokens'],
        // Change methods can modify the state, but you don't receive the returned value when called
        changeMethods: ['transfer_deposit_for_nft'],
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
        contract.get_all_locked_tokens({}).then(res => {
            const nfts = getNFTsLockedInfo(res, true);
            console.log(nfts);
            showGallery(nfts);
        });

    }
}

function getNFTsLockedInfo(res, isLocked) {
    let nfts = [];
    for (let el of res) {
        const json = el['json_token'];
        const locked = el['locked_token'];
        const image_url = json['metadata']['media'];
        const title = json['metadata']['title'] || "No title";

        const curNFT = new LockedNFT(
            new NFT(title, json['owner_id'], json['token_id'], image_url, isLocked),
            locked['apr'], locked['borrowed_money'], locked['duration'], locked['owner_id']
        );

        nfts.push(curNFT);
        market_nfts[json['token_id']] = curNFT;
    }
    return nfts
}

function showGallery(nfts) {
    for (let i = 0; i < nfts.length; i++) {
        document.querySelector('.gallery').innerHTML += showNFT(nfts[i]);
    }

    if (nfts.length > 0) {
        $('.container_image').click(function () {
            showModalNftLocked(this.id)
        });
    }
}

function showModalNftLocked(id) {
    modalLockedNFT.style.display = "block";
    const nft = market_nfts[id];
    const deposit = 1;
    document.querySelector('.apr').innerHTML = nft.apr;
    document.querySelector('.days').innerHTML = nft.duration;
    document.querySelector('.amount').innerHTML = formatNearAmount(nft.borrowed_money);

    $('.transfer-money').click(function () {
        contract.transfer_deposit_for_nft({token_id: nft.NFT.token_id}, GAS, nft.borrowed_money).then(updateUI);
    });
}


document.querySelector('.closeNFT').addEventListener("click", function () {
    modalLockedNFT.style.display = "none";
});

function showNFT(nft) {
    const div_info = `class=\"container_image\" id=\"${nft.NFT.token_id}\"`;
    return "<div class=\"nft\">\n" +
        "   <div class=\"nft__image\"><img " + div_info + " src=\"" + nft.NFT.url + "\" alt=\"" + nft.NFT.title + "\"></div>\n" +
        "   <h2 class=\"nft__title\">" + nft.NFT.title + "</h2>\n" +
        "   <p class=\"nft__owner\">" + nft.owner_id + "</p>\n" +
        "</div>"
}

let modalLockedNFT = document.querySelector(".modalNftLocked");

window.nearInitPromise = connect(nearConfig).then(updateUI);