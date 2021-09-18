import "regenerator-runtime/runtime";
import * as nearAPI from "near-api-js";
import getConfig from "./config";
import {getNFTsInfo, showNFT} from "./nft-utils.js"

const nearConfig = getConfig(process.env.NODE_ENV || "development");

const GAS = "200000000000000";

export const {
    utils: {
        format: {
            formatNearAmount, parseNearAmount
        }
    }
} = nearAPI;

let allNfts = {};
let marketNfts = {};
let myLoanNFTs = {};
let navigatorState = "Market";

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
        viewMethods: ['get_debtors_tokens', 'nft_tokens_for_owner', 'get_locked_tokens', 'get_all_locked_tokens'],
        // Change methods can modify the state, but you don't receive the returned value when called
        changeMethods: ['nft_mint', 'transfer_nft_to_contract', 'transfer_nft_back', 'repaid_loan',
            'transfer_deposit_for_nft', 'check_transfer_overdue_nft_to_creditor'],
        // Sender is the account ID to initialize transactions.
        // getAccountId() will return empty string if user is still unauthorized
        sender: window.walletConnection.getAccountId()
    });

}


function updateUI() {
    console.log("update UI");

    document.querySelector('.'+navigatorState.toLowerCase()+'-btn').style.textDecoration = "underline";
    if (!window.walletConnection.getAccountId()) {
        document.querySelector('.my-account').style.display = 'none';
        document.querySelector('.my-karma').style.display = 'none';
        document.querySelector('.login').innerHTML = "sign in";
    } else {
        let myAcc = document.querySelector('.my-account');
        let myKarma = document.querySelector('.my-karma');
        myAcc.style.display = 'block';
        myAcc.innerHTML = window.walletConnection.getAccountId();
        myKarma.style.display = 'block';
        myKarma.innerHTML = "Karma: 100";

        document.querySelector('.login').innerHTML = "sign out";
        document.getElementsByClassName("gallery")[0].innerHTML = "";

        contract.nft_tokens_for_owner({
            account_id: window.walletConnection.getAccountId(),
            from_index: '0',
            limit: '50'
        }).then(res => {
            const nfts = getNFTsInfo(res, false);
            addLoadedNfts(nfts,false);
            if (navigatorState === "MyNFTs") {
                showGallery(nfts, false);
            }
        });

        contract.get_locked_tokens({
            account_id: window.walletConnection.getAccountId(),
            need_all: true
        }).then(res => {
            const nfts = getNFTsInfo(res, true);
            addLoadedNfts(nfts, false);
            if (navigatorState === "MyNFTs") {
                showGallery(nfts, false);
            }
        });

        contract.get_all_locked_tokens({}).then(res => {
            const nfts = getNFTsInfo(res, true);
            addLoadedNfts(nfts, true);
            if (navigatorState === "Market") {
                showGallery(nfts, true);
            }
        });


        contract.get_debtors_tokens({
            account_id: window.walletConnection.getAccountId()
        }).then(res => {
            const nfts = getNFTsInfo(res, true);
            if (navigatorState === "MyLoans") {
                if (nfts.length === 0){
                    const alertMsg = "You have no debtors";
                    document.querySelector(".gallery").innerHTML = "<p class=\'alert\'>" + alertMsg + "</p>";
                } else {
                    showGallery(nfts, false);
                }
            }
        });

    }
}

function addLoadedNfts(nfts, isMarket){
    for (let nft of nfts){
        if (isMarket) {
            marketNfts[nft.token_id] = nft;
        } else {
            allNfts[nft.token_id] = nft;
        }
    }
}


function showGallery(nfts, isMarket) {
    for (let i = 0; i < nfts.length; i++) {
        if (isMarket) {
            marketNfts[nfts[i].token_id] = nfts[i];
        } else {
            allNfts[nfts[i].token_id] = nfts[i];
        }
        document.querySelector(".gallery").innerHTML += showNFT(nfts[i], isMarket);
    }
    if (nfts.length > 0) {
        $('.container_image').off('click').click(function () {
            showModalNft(this.id, isMarket)
        });
    }
}

function showModalNft(id, isMarket) {
    modalNFT.style.display = "block";
    let nft;
    if (isMarket) {
        nft = marketNfts[id];
    } else {
        nft = allNfts[id];
    }
    let deposit = parseNearAmount('0.1');
    const lockedBlock = document.getElementById('modal-back-block');
    const borrowBlock = document.getElementById('modal-borrow-block');
    if (nft.isLocked) {
        document.querySelector('.title-modal-nft').innerHTML = "Return";
        lockedBlock.style.display = 'block';
        borrowBlock.style.display = 'none';

        document.querySelector('.apr').innerHTML = nft.apr;
        document.querySelector('.duration').innerHTML = nft.duration;
        document.querySelector('.amount').innerHTML = formatNearAmount(nft.borrowed_money);
        if (nft.is_confirmed) {
            document.querySelector('.confirmed').style.display = 'block';
            document.querySelector('.modal-main-btn').innerHTML = "Repaid loan";
            document.querySelector('.creditor').innerHTML = nft.creditor;
            let curTime = Math.round(new Date().getTime() / 1000);
            console.log(curTime - Number.parseInt((nft.start_time).toString().slice(0,10)));
            let timeLeft = Number.parseInt(nft.duration) - (curTime - Number.parseInt((nft.start_time).toString().slice(0,10)));
            document.querySelector('.time-left').innerHTML = timeLeft > 0 ? timeLeft: "Time is over";

            console.log(nft.borrowed_money, nft.apr);
            let multiplier = 1 + (Number.parseInt(nft.apr) / 100);
            console.log(multiplier);
            deposit = parseNearAmount((Number.parseFloat(formatNearAmount(nft.borrowed_money)) * multiplier).toString());
            console.log(deposit);
            $('.modal-main-btn').off('click').click(function () {
                contract.repaid_loan({token_id: id}, GAS, deposit.toString()).then(updateUI);
            });
        } else {
            document.querySelector('.confirmed').style.display = 'none';
            if (isMarket){
                document.querySelector('.modal-main-btn').innerHTML = "Lend";
                $('.modal-main-btn').off('click').click(function () {
                    contract.transfer_deposit_for_nft({token_id: nft.token_id}, GAS, nft.borrowed_money).then(updateUI);
                });
            } else {
                document.querySelector('.modal-main-btn').innerHTML = "Return NFT";
                $('.modal-main-btn').off('click').click(function () {
                    contract.transfer_nft_back({token_id: id}, GAS, deposit).then(updateUI);
                });
            }
        }
    } else {
        document.querySelector('.modal-main-btn').innerHTML = "Place offer";
        lockedBlock.style.display = 'none';
        borrowBlock.style.display = 'block';
        $('.modal-main-btn').off('click').click(function () {
            const amount = parseNearAmount(document.querySelector(".input-amount").value);
            console.log(amount);
            const days = Number.parseInt(document.querySelector(".input-duration").value);
            const apr = Number.parseInt(document.querySelector(".input-apr").value);

            if (amount && days && apr) {
                const params = {token_id: id, borrowed_money: amount, apr: apr, borrow_duration: days};
                contract.transfer_nft_to_contract(params, GAS, deposit).then(updateUI);
                modalNFT.style.display = "none";
            }
        });
    }
}

function changeNavigatorState(newState){
    // if (window.walletConnection.getAccountId()) {
    //     if (navigatorState !== newState) {
    //         navigatorState = newState;
    //         updateUI()
    //     }
    // } else if (navigatorState !== 'Market'){
    //
    // }
    if (navigatorState !== newState) {
        document.querySelector('.'+navigatorState.toLowerCase()+'-btn').style.textDecoration = "none";
        navigatorState = newState;
        updateUI()
    }
}




function getMetadata(title, media) {
    return {
        title: title,
        media: media,
        issued_at: Date.now().toString()
    };
}

let modalMint = document.getElementById("mintModal");
let modalNFT = document.getElementById("nftModal");


// document.querySelector('.check-overdue').addEventListener("click", function () {
//     contract.check_transfer_overdue_nft_to_creditor({token_id: "token-1631987533962"}).then(updateUI);
// });


document.querySelector('.closeMint').addEventListener("click", function () {
    modalMint.style.display = "none";
});

document.querySelector('.closeNFT').addEventListener("click", function () {
    modalNFT.style.display = "none";
});

document.querySelector('.mint').addEventListener("click", function () {
    console.log("try mint");
    const title = document.getElementsByClassName("mint-title")[0].value;
    const url = document.getElementsByClassName("mint-url")[0].value;

    if (title && url) {
        const deposit = parseNearAmount('0.1');
        const metadata = getMetadata(title, url);
        const royality = null;
        contract.nft_mint({
            token_id: 'token-' + Date.now(),
            metadata,
            royality
        }, GAS, deposit).then(updateUI);
        modalMint.style.display = "none";
    }
});

document.querySelector('.mint-btn').addEventListener("click", function () {
    modalMint.style.display = "block";
});


document.querySelector('.market-btn').addEventListener("click", function () {
    changeNavigatorState("Market");
});

document.querySelector('.mynfts-btn').addEventListener("click", function () {
    changeNavigatorState("MyNFTs");
});

document.querySelector('.myloans-btn').addEventListener("click", function () {
    changeNavigatorState("MyLoans");
});


document.querySelector('.login').addEventListener("click", function () {
    if (!window.walletConnection.getAccountId()) {
        walletConnection.requestSignIn(nearConfig.contractName, 'My contr');
    } else {
        walletConnection.signOut();
    }
    updateUI()
});


window.nearInitPromise = connect(nearConfig).then(updateUI);

