const nearAPI = require("near-api-js");

const { connect, keyStores ,KeyPair } = nearAPI;

async function auth(){
    const keyStore = new keyStores.InMemoryKeyStore();
    const PRIVATE_KEY =
    "2Ng7sBXHcxX2RtBZs543gQsSsFxg2Mz4oWF5n8UKZD3tBkrr4faPGkUQMXg6euo32PSTKodXhicbgTWU7JVGBHxp";
    const keyPair = KeyPair.fromString(PRIVATE_KEY);

    await keyStore.setKey("testnet", "contract.pep.testnet", keyPair);
    console.log("AUTH START")
    const config = {
        networkId: "testnet",
        keyStore: keyStore,
        nodeUrl: "https://rpc.testnet.near.org",
        walletUrl: "https://wallet.testnet.near.org",
        helperUrl: "https://helper.testnet.near.org",
        explorerUrl: "https://explorer.testnet.near.org",
    };

    const near = await connect(config);
    const account = await near.account("contract.pep.testnet");
    console.log("AUTH END")
    return account;
}

async function main() {
    const account = await auth();
    console.log("DONE")

    read_contract(account);

    // contractId = "sber.mintspace2.testnet";
    // tokenId = "1";
    // receiverId = "moderator.testnet";
    // send_nft(account, contractId, tokenId, receiverId);
}

async function read_contract(account) {
    try {
        const tx = await account.viewFunction(
            'con.alevoro.testnet',
            'get_all_locked_tokens'
        )
        console.log(tx)
        console.log("STATUS:", tx.status)
    } catch (e) {
        console.log("ERROR");
        console.log(e);
    }
}

async function send_nft(account, contractId, tokenId, receiverId) {
    try {
        const tx = await account.functionCall(
            contractId,
            'nft_transfer',
            {
                token_id: tokenId,
                receiver_id: receiverId
            },
            '100000000000000',
            '1'
        )
        console.log("STATUS:", tx.status)
    } catch (e) {
        console.log("ERROR");
        console.log(e);
    }
}

main();