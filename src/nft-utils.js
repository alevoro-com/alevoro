import {LockedNFT, NFT} from "./classes";
export {getNFTsInfo, showNFT}


function getNFTsInfo(res) {
    let nfts = [];
    console.log("LOCKED");
    for (let el of res) {
        console.log(el);
        // let nft_token = el;
        // let locked_info = null;
        //
        //
        // const title = nft_token['metadata']['title'] || "No title";
        // const owner_id = nft_token['owner_id'];
        // const token_id = nft_token['token_id'];
        // const image_url = nft_token['metadata']['media'];
        //
        // let curNFT = new NFT(title, owner_id, token_id, image_url, isLocked);
        // if (isLocked) {
        //     console.log(locked_info);
        //     curNFT = new LockedNFT(title, owner_id, token_id, image_url, isLocked,
        //         locked_info['apr'], locked_info['borrowed_money'], locked_info['duration'],
        //         locked_info['owner_id'],locked_info['is_confirmed'], locked_info['creditor'],
        //         locked_info['start_time'])
        // }
        //
        // nfts.push(curNFT);
    }
    console.log("LOCKED END");
    return nfts
}


function showNFT(nft, nftState) {
    const divInfo = `class=\"container_image\" id=\"${nft.token_id}\"`;
    let bottomText = nft.owner;
    if (nftState === 'Market' || nftState === 'MyLoans') {
        bottomText = nft.real_owner;
    } else if (nft.isLocked) {
        if (nft.is_confirmed) {
            bottomText = "Collateral";
        } else {
            bottomText = "Locked";
        }
    }
    return "<div class=\"nft\">\n" +
        "   <div class=\"nft__image\"><img " + divInfo + " src=\"" + nft.url + "\" alt=\"" + nft.title + "\"></div>\n" +
        "   <a class=\"nft__title\"  href=\"" + nft.extra +"\" target=\"_blank\">" + nft.title + "</a>\n" +
        "   <p class=\"nft__owner\">" + bottomText + "</p>\n" +
        "</div>"
}
