import "regenerator-runtime/runtime";
import * as nearAPI from "near-api-js";
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

class NFT {
  constructor(title, owner, token_id, url, isLocked) {
      this.title = title;
      this.owner = owner;
      this.token_id = token_id;
      this.url = url;
      this.isLocked = isLocked;
  }

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
    document.getElementsByClassName("gallery")[0].innerHTML = "";
    
    contract.nft_tokens_for_owner({
      account_id: window.walletConnection.getAccountId(),
      from_index: '0',
      limit: '50'
    }).then(res => {
      const nfts = getNFTsInfo(res, false);
      showGallery(nfts);
    });

    contract.get_locked_tokens({
      account_id: window.walletConnection.getAccountId()
    }).then(res => {
      const nfts = getNFTsInfo(res, true);
      showGallery(nfts);
    });

  }
}

function getNFTsInfo(res, isLocked){
  let nfts = [];
  for (let el of res) {
    console.log(el['token_id']);
    const image_url = el['metadata']['media'];
    const title = el['metadata']['title'] || "No title";
    nfts.push(new NFT(title, el['owner_id'],el['token_id'], image_url, isLocked));
    all_nfts[el['token_id']] = new NFT(title, el['owner_id'],el['token_id'], image_url, isLocked);
  }
  return nfts
}

function showGallery(nfts){
  for (let i = 0; i < nfts.length; i++) {
    document.getElementsByClassName("gallery")[0].innerHTML += showNFT(nfts[i]);
  }
  if (nfts.length > 0){
    $('.container_image').click(function(){
      showModalNft(this.id)
    });
  }
}

function showModalNft(id){
  modalNFT.style.display = "block";
  const nft = all_nfts[id];
  const deposit = 1;
  const lockedBlock = document.getElementById('modal-back-block');
  const borrowBlock = document.getElementById('modal-borrow-block');
  if (nft.isLocked) {
    document.getElementsByClassName('title-modal-nft')[0].innerHTML = "Return";
    lockedBlock.style.display = 'block';
    borrowBlock.style.display = 'none';
    $('.transfer-nft-back').click(function(){
      contract.transfer_nft_back({ token_id: id}, GAS, deposit).then(updateUI);
    });
  } else {
    document.getElementsByClassName('title-modal-nft')[0].innerHTML = "Borrow";
    lockedBlock.style.display = 'none';
    borrowBlock.style.display = 'block';
    $('.transfer-nft').click(function(){
      const amount = Number.parseInt(document.getElementsByClassName("input-amount")[0].value);
      const days = Number.parseInt(document.getElementsByClassName("input-days")[0].value);
      const apr = Number.parseInt(document.getElementsByClassName("input-apr")[0].value);
    
      if (amount && days && apr) {
        const params = { token_id: id, borrowed_money: amount, apr: apr, borrow_duration: days};
        contract.transfer_nft_to_contract(params, GAS, deposit).then(updateUI);
        modalNFT.style.display = "none";
      }
    });
  }
}


function showNFT(nft){
  const div_info = `class=\"container_image\" id=\"${nft.token_id}\"`;
  const bottom = nft.owner === window.walletConnection.getAccountId() ? nft.owner : "Locked: " + nft.owner;
  return "<div class=\"nft\">\n" +
          "   <div class=\"nft__image\"><img "+div_info+" src=\""+nft.url+"\" alt=\""+nft.title+"\"></div>\n" +
          "   <h2 class=\"nft__title\">"+nft.title+"</h2>\n" +
          "   <p class=\"nft__owner\">"+bottom+"</p>\n" +
          "</div>"
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


document.querySelector('.open-mint').addEventListener("click", function() {
  modalMint.style.display = "block";
});

document.querySelector('.increase').addEventListener("click", function() {
  contract.increment({account_id: window.walletConnection.getAccountId()}).then(updateUI);
});

document.querySelector('.closeMint').addEventListener("click", function() {
  modalMint.style.display = "none";
});

document.querySelector('.closeNFT').addEventListener("click", function() {
  modalNFT.style.display = "none";
});

document.querySelector('.mint').addEventListener("click", function() {
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



document.querySelector('.login').addEventListener("click", function() {
  if (!window.walletConnection.getAccountId()) {
    walletConnection.requestSignIn(nearConfig.contractName, 'My contr');
  }else{
    walletConnection.signOut();
  }
  updateUI()
});


window.nearInitPromise = connect(nearConfig).then(updateUI);

