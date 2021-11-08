const nearAPI = require("near-api-js");

const { connect, keyStores ,KeyPair } = nearAPI;

const CONTRACT_NAME = 'contract.alevoro.testnet';

async function auth(){
    const keyStore = new keyStores.InMemoryKeyStore();
    const PRIVATE_KEY =
    "PRIVATE KEY HERE";
    const keyPair = KeyPair.fromString(PRIVATE_KEY);

    await keyStore.setKey("testnet", "contract.alevoro.testnet", keyPair);
    console.log("AUTH START");
    const config = {
        networkId: "testnet",
        keyStore: keyStore,
        nodeUrl: "https://rpc.testnet.near.org",
        walletUrl: "https://wallet.testnet.near.org",
        helperUrl: "https://helper.testnet.near.org",
        explorerUrl: "https://explorer.testnet.near.org",
    };

    const near = await connect(config);
    const account = await near.account("contract.alevoro.testnet");
    console.log("AUTH END");
    return account;
}

async function main() {
    const account = await auth();
    console.log("DONE");

    const nfts = await read_contract(account);
    for (let nft of nfts) {
        const idAndContract = nft['token_id'].split(":");
        if (nft.state === "Return") {
            await send_nft(account, idAndContract[1], idAndContract[0], nft.owner_id)
        } else if (nft.state === "TransferToBorrower") {
            await send_nft(account, idAndContract[1], idAndContract[0], nft.owner_id)
        } else if (nft.state === "TransferToCreditor") {
            await send_nft(account, idAndContract[1], idAndContract[0], nft.creditor)
        }
    }


}

async function read_contract(account) {
    try {
        const tx = await account.viewFunction(
            CONTRACT_NAME,
            'get_all_locked_tokens',
            {
                need_all: true
            }
        );
        console.log(tx);
        return tx
    } catch (e) {
        console.log("ERROR");
        console.log(e);
        return []
    }
}

async function send_nft(account, contractId, tokenId, receiverId) {
    try {
        const tx = await account.viewFunction(
            contractId,
            'nft_token',
            {
                token_id: tokenId,
            }
        );
        if (tx.owner_id.Account === receiverId) {
            console.log(tx);
            deleteFromContract(account, contractId, tokenId);
        } else {
            const res = await account.functionCall(
                contractId,
                'nft_transfer',
                {
                    token_id: tokenId,
                    receiver_id: receiverId
                },
                '100000000000000',
                '1'
            );
            if (res.status !== null && typeof res.status.SuccessValue === 'string') {
                deleteFromContract(account, contractId, tokenId);
            }
        }
    } catch (e) {
        console.log("ERROR");
        console.log(e);
    }
}

async function deleteFromContract(account, contractId, tokenId){
    console.log("DEL");
    const del_tx = await account.functionCall(
        CONTRACT_NAME,
        'remove_transferred_token_from_locked_tokens',
        {
            token_id: tokenId + ":" + contractId,
        },
        '100000000000000',
        '1'
    );
    console.log(del_tx);
}

main();