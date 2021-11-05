import {LockedNFT} from "./classes";
export {getNFTsInfo, showNFT}


function getNFTsInfo(res, con_name) {
    let nfts = [];
    console.log("LOCKED");
    for (let nft of res) {
        console.log(nft);
        let curNFT = new LockedNFT(nft['title'], con_name, nft['token_id'], nft['media'], nft['extra'], nft['type'],
            true, nft['apr'], nft['borrowed_money'], nft['duration'], nft['owner_id'], nft['state'],
            nft['creditor'], nft['start_time']);

        nfts.push(curNFT);
    }
    console.log("LOCKED END");
    console.log(nfts);
    return nfts
}


function showNFT(nft, nftState) {
    console.log("show");
    const divInfo = `class=\"container_image\" id=\"${nft.token_id}\"`;
    let bottomText = nft.owner;
    if (nftState === 'Market' || nftState === 'MyLoans') {
        bottomText = nft.real_owner;
    } else if (nft.isLocked) {
        if (nft.state === "Sale"){
            bottomText = "Locked";
        } else if (nft.state === "Locked") {
            bottomText = "Collateral";
        }
    }
    return "<div class=\"nft\">\n" +
        "   <div class=\"nft__image\"><img " + divInfo + " src=\"" + nft.url + "\" alt=\"" + nft.title + "\"></div>\n" +
        "   <a class=\"nft__title\"  href=\"" + nft.extra +"\" target=\"_blank\">" + nft.title + "</a>\n" +
        "   <p class=\"nft__owner\">" + bottomText + "</p>\n" +
        "</div>"
}
