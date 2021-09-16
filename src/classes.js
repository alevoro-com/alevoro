export {NFT, LockedNFT}

class NFT {
    constructor(title, owner, token_id, url, isLocked) {
        this.title = title;
        this.owner = owner;
        this.token_id = token_id;
        this.url = url;
        this.isLocked = isLocked;
    }

}


class LockedNFT {
    constructor(NFT, apr, borrowed_money, duration, owner_id) {
        this.NFT = NFT;
        this.apr = apr;
        this.borrowed_money = borrowed_money;
        this.duration = duration;
        this.owner_id = owner_id;
    }

}


