module.exports = {getNFTs, viewAccountNFT};

import {getMintbase, mintbaseCard} from "./mintbase";
const nearApi = require("near-api-js");


async function getNFTs(accountId) {
    try {
        const res = await fetch('https://helper.' + (accountId.substr(-5) === '.near' ? 'mainnet' : 'testnet') + '.near.org/account/' + accountId + '/likelyNFTs', {timeout: 30000});
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
        const result = [];

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
        const urlData = {}, urlPtr = {};
        for (const id of list) {
            const url = await account.viewFunction(contractId, 'nft_token_uri', {token_id: '' + id});
            if (url && url.error) continue;
            const data = urlData[url] ? urlData[url] : await getMintbase(url);
            if (data && !data.error) {
                urlData[url] = data;
                if (urlPtr[url] === undefined) {
                    urlPtr[url] = result.length;
                    result.push({contract: contractId, ...mintbaseCard(urlData[url]), id: id})
                } else {
                    const nid = result[urlPtr[url]].id + ',' + id;
                    result[urlPtr[url]] = {contract: contractId, ...mintbaseCard(urlData[url], nid), id: nid}
                }
            }
        }
        return result
    } catch (err) {
        console.log(err);
        return {error: err.type || err}
    }
}
