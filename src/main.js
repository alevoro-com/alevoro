import "regenerator-runtime/runtime";
import * as nearAPI from "near-api-js";
import getConfig from "./config";
import {getNFTsInfo, showNFT} from "./nft-utils.js";

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

    document.querySelector('.' + navigatorState.toLowerCase() + '-btn').style.textDecoration = "underline";
    document.querySelector('.gallery-alert').style.display = 'block';
    document.querySelector('.alert').innerHTML = "";
    if (!window.walletConnection.getAccountId()) {
        document.querySelector('.alert').innerHTML = "Please, sign in";
        document.querySelector('.my-account').style.display = 'none';
        document.querySelector('.my-karma').style.display = 'none';
        document.querySelector('.login').innerHTML = "sign in";
        document.querySelector(".gallery").innerHTML = "";
    } else {
        let myAcc = document.querySelector('.my-account');
        let myKarma = document.querySelector('.my-karma');
        myAcc.style.display = 'block';
        myAcc.innerHTML = window.walletConnection.getAccountId();
        myKarma.style.display = 'block';
        myKarma.innerHTML = "Karma: 100";

        document.querySelector('.login').innerHTML = "sign out";
        document.querySelector(".gallery").innerHTML = "";

        contract.nft_tokens_for_owner({
            account_id: window.walletConnection.getAccountId(),
            from_index: '0',
            limit: '50'
        }).then(res => {
            initNFTs(res, false, "MyNFTs");
        });

        contract.get_locked_tokens({
            account_id: window.walletConnection.getAccountId(),
            need_all: true
        }).then(res => {
            initNFTs(res, true, "MyNFTs");
        });

        contract.get_all_locked_tokens({}).then(res => {
            initNFTs(res, true, "Market");
        });


        contract.get_debtors_tokens({
            account_id: window.walletConnection.getAccountId()
        }).then(res => {
            initNFTs(res, true, "MyLoans");
        });

        setTimeout(function () {
            document.querySelector('.alert').innerHTML = getAlertPhrase();
        }, 500);

    }
}

function initNFTs(res, isLocked, PAGE) {
    const nfts = getNFTsInfo(res, isLocked);
    addLoadedNfts(nfts, PAGE);
    if (navigatorState === PAGE) {
        if (nfts.length > 0){
            document.querySelector('.gallery-alert').style.display = 'none';
        }
        showGallery(nfts, PAGE);
    }
}

function addLoadedNfts(nfts, curPage) {
    for (let nft of nfts) {
        if (curPage === "Market") {
            marketNfts[nft.token_id] = nft;
        } else if (curPage === "MyLoans") {
            myLoanNFTs[nft.token_id] = nft;
        } else {
            allNfts[nft.token_id] = nft;
        }
    }
}

function getAlertPhrase(){
    if (navigatorState === 'Market'){
        return 'Market is empty'
    }
    if (navigatorState === 'MyNFTs'){
        return "You don't have any NFTs"
    }
    if (navigatorState === 'MyLoans'){
        return "You don't have any debtors"
    }
    return ""
}


function showGallery(nfts, nftState) {
    for (let i = 0; i < nfts.length; i++) {
        document.querySelector(".gallery").innerHTML += showNFT(nfts[i], nftState);
    }
    if (nfts.length > 0) {
        $('.container_image').off('click').click(function () {
            showModalNft(this.id, nftState)
        });
    }
}

function showModalNft(id, nftState) {
    modalNFT.style.display = "block";
    let nft;
    if (nftState === 'Market') {
        nft = marketNfts[id];
    } else if (nftState === 'MyLoans') {
        nft = myLoanNFTs[id];
    } else {
        nft = allNfts[id];
    }
    let deposit = parseNearAmount('0.1');
    const lockedBlock = document.getElementById('modal-back-block');
    const borrowBlock = document.getElementById('modal-borrow-block');
    document.querySelector('.modal-main-btn').style.display = 'inline';
    if (nft.isLocked) {
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
            let timeLeft = Number.parseInt(nft.duration) - (curTime - Number.parseInt((nft.start_time).toString().slice(0, 10)));

            let multiplier = 1 + (Number.parseInt(nft.apr) / 100);
            deposit = parseNearAmount((Number.parseFloat(formatNearAmount(nft.borrowed_money)) * multiplier).toString());
            if (nftState === 'MyLoans') {
                document.querySelector('.title-modal-nft').innerHTML = "Debtor";
                if (timeLeft > 0) {
                    showTimer(timeLeft, function () {
                        if (navigatorState === 'MyLoans') {
                            document.querySelector('.modal-main-btn').style.display = 'inline';
                            document.querySelector('.modal-main-btn').innerHTML = "Claim NFT";
                            $('.modal-main-btn').off('click').click(function () {
                                document.querySelector('.modal-main-btn').style.display = 'none';
                                contract.check_transfer_overdue_nft_to_creditor({token_id: id}).then(goToNFTsAndUpdate);
                            });
                        }
                    });
                    document.querySelector('.modal-main-btn').style.display = 'none';
                } else {
                    showTimer(timeLeft, () => {});
                    document.querySelector('.modal-main-btn').innerHTML = "Claim NFT";
                    $('.modal-main-btn').off('click').click(function () {
                        document.querySelector('.modal-main-btn').style.display = 'none';
                        contract.check_transfer_overdue_nft_to_creditor({token_id: id}).then(goToNFTsAndUpdate);
                    });
                }
            } else {
                document.querySelector('.title-modal-nft').innerHTML = "Repay";
                showTimer(timeLeft, function () {
                    if (navigatorState === 'MyNFTs') {
                        document.querySelector('.modal-main-btn').style.display = 'none';
                        document.querySelector('.title-modal-nft').innerHTML = "Lost NFT";
                    }
                });
                $('.modal-main-btn').off('click').click(function () {
                    contract.repaid_loan({token_id: id}, GAS, deposit.toString()).then(updateUI);
                });
            }
        } else {
            document.querySelector('.confirmed').style.display = 'none';
            if (nftState === 'Market') {
                if (window.walletConnection.getAccountId() === nft.real_owner) {
                    document.querySelector('.title-modal-nft').innerHTML = "Your NFT";
                    document.querySelector('.modal-main-btn').style.display = 'none';
                } else {
                    document.querySelector('.title-modal-nft').innerHTML = "Lend";
                    document.querySelector('.modal-main-btn').innerHTML = "Lend";
                    $('.modal-main-btn').off('click').click(function () {
                        contract.transfer_deposit_for_nft({token_id: nft.token_id}, GAS, nft.borrowed_money).then(updateUI);
                    });
                }
            } else {
                document.querySelector('.title-modal-nft').innerHTML = "Return";
                document.querySelector('.modal-main-btn').innerHTML = "Return NFT";
                $('.modal-main-btn').off('click').click(function () {
                    contract.transfer_nft_back({token_id: id}, GAS, deposit).then(updateUI);
                });
            }
        }
    } else {
        document.querySelector('.title-modal-nft').innerHTML = "Borrow";
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

function showTimer(secondsLeft, callback) {
    calculate();
    let timer = setInterval(calculate, 1000);

    function calculate() {
        function formatNumber(num) {
            if (num / 10 < 1){
                return '0' + num;
            }
            return num;

        }
        if (secondsLeft <= 0){
            clearInterval(timer);
            document.querySelector('.timer').innerHTML = 'Time is over';
            callback();
            return
        }
        const seconds = formatNumber(secondsLeft % 60);
        const minutes = formatNumber(Math.floor(secondsLeft / 60) % 60);
        const hours = formatNumber(Math.floor(secondsLeft / (60 * 60)) % 60);
        const days = Math.floor(secondsLeft / (60 * 60 * 24)) % 60;

        document.querySelector('.timer').innerHTML = `${days} days, ${hours}:${minutes}:${seconds}`;

        secondsLeft -= 1;
    }
}

function changeNavigatorState(newState) {
    if (navigatorState !== newState) {
        document.querySelector('.' + navigatorState.toLowerCase() + '-btn').style.textDecoration = "none";
        navigatorState = newState;
        updateUI()
    }
}

function goToNFTsAndUpdate() {
    modalNFT.style.display = "none";
    changeNavigatorState("MyNFTs");
}


function getMetadata(title, media) {
    return {
        title: title,
        media: media,
        issued_at: Date.now().toString()
    };
}

document.querySelector('.mint-url').addEventListener("input", function (e) {
    document.querySelector('.container-mint-image').innerHTML =
        "   <div class=\"mint-image\"><img class=\"container_image\" src=\"" + e.target.value + "\"\></div>\n";
});

let modalMint = document.getElementById("mintModal");
let modalNFT = document.getElementById("nftModal");


document.querySelector('.closeMint').addEventListener("click", function () {
    modalMint.style.display = "none";
});

document.querySelector('.closeNFT').addEventListener("click", function () {
    modalNFT.style.display = "none";
});

document.querySelector('.modal-mint-btn').addEventListener("click", function () {
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
    document.querySelector('.container-mint-image').innerHTML = "";
    document.querySelector('.mint-url').value = '';
    document.querySelector('.mint-title').value = '';
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

