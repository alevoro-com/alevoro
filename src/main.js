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
    document.getElementsByClassName("gallery")[0].innerHTML = "";
    contract.nft_tokens_for_owner({
      account_id: window.walletConnection.getAccountId(),
      from_index: '0',
      limit: '50'
    }).then(res => {
      const nfts = getNFTsInfo(res);
      showGallery(nfts);
    });

    contract.get_locked_tokens({
      account_id: window.walletConnection.getAccountId()
    }).then(res => {
      const nfts = getNFTsInfo(res);
      showGallery(nfts);
    });

  }
}

function getNFTsInfo(res){
  let nfts = [];
  for (let el of res) {
    console.log(el['token_id']);
    const image_url = el['metadata']['media'];
    const title = el['metadata']['title'] || "No title";
    nfts.push([title, el['owner_id'], image_url])
  }
  return nfts
}

function showGallery(nfts){
  for (let nft of nfts) {
    document.getElementsByClassName("gallery")[0].innerHTML += showNFT(nft);
  }
}

function showNFT(nft){
  return "<div class=\"nft\">\n" +
          "   <div class=\"nft__image\"><img class=\"container_image\" src=\""+nft[2]+"\" alt=\""+nft[0]+"\"></div>\n" +
          "   <h2 class=\"nft__title\">"+nft[0]+"</h2>\n" +
          "   <p class=\"nft__owner\">"+nft[1]+"</p>\n" +
          "</div>"
}

function getMetadata(title, media) {
  return {
    title: title,
    media: media,
    issued_at: Date.now().toString()
  };
}

let modal = document.getElementById("myModal");

document.querySelector('.open-mint').addEventListener("click", function() {
  modal.style.display = "block";
});

document.querySelector('.increase').addEventListener("click", function() {
  contract.increment({account_id: window.walletConnection.getAccountId()}).then(updateUI);
});

document.querySelector('.transfer-nft').addEventListener("click", function() {
  const deposit = 1;
  const params = { creditor_id: "biba7.testnet", token_id: "token-1631554286914", lend_money: 2, apr: 228, lend_duration: 1488}
  contract.transfer_nft_to_contract(params, GAS, deposit).then(updateUI);
});

document.querySelector('.transfer-back').addEventListener("click", function() {
  // const deposit = 1;
  // const params = { creditor_id: "biba7.testnet", token_id: "token-1631554286914", lend_money: 2, apr: 228, lend_duration: 1488}
  // contract.transfer_nft_to_contract(params, GAS, deposit).then(updateUI);
});

document.querySelector('.close').addEventListener("click", function() {
  modal.style.display = "none";
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
    modal.style.display = "none";
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

