module.exports = {getNFTs, viewAccountNFT};

import {getMintbase, mintbaseCard} from "./mintbase";
const nearApi = require("near-api-js");


async function getNFTs(accountId) {
    try {
        const res = await fetch('https://helper.' + (accountId.substr(-5) === '.near' ? 'mainnet' : 'testnet')
            + '.near.org/account/' + accountId + '/likelyNFTs', {timeout: 30000});
        if (res.status < 199 || res.status > 299) {
            return {error: res.statusText + ' (' + res.status + ')'}
        }
        const text = await res.text();
        try {
            return JSON.parse(text)
        } catch (err) {
            return {error: text}
        }
    } catch (err) {
        return {error: err}
    }
}



async function viewAccountNFT(contractId, accountId) {
    try {
        let result = [];

        const network = accountId.substr(-5) === '.near' ? 'mainnet' : 'testnet';
        const provider = new nearApi.providers.JsonRpcProvider('https://rpc.' + network + '.near.org');
        const account = new nearApi.Account({provider: provider});

        // MINTBASE
        const list = await account.viewFunction(contractId, 'nft_tokens_for_owner_set', {
            account_id: accountId,
            from_index: '0',
            limit: 100
        });
        if (list.error) return list;
        for (const id of list) {
            const url = await account.viewFunction(contractId, 'nft_token_uri', {token_id: '' + id});
            const data_specific = await getMintbase(url);
            if (data_specific && !data_specific.error){
                let cur_res = mintbaseCard(data_specific, true);
                const data = await account.viewFunction(contractId, 'nft_token', {token_id: '' + id});
                if (data) {
                    const metadata = mintbaseCard(data, false);
                    const reference = "https://" + (network === 'testnet' ? "testnet." : "") +
                        "mintbase.io/thing/" + metadata['ref'] + ":" + contractId;
                    cur_res = {
                        ...cur_res,
                        owner_id: metadata['owner_id'],
                        id: id + ":" + contractId,
                        reference: reference
                    };
                    result.push(cur_res);
                }
            }

        }
        return result
    } catch (err) {
        console.log(err);
        return {error: err.type || err}
    }
}
