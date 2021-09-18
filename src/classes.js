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



class LockedNFT extends NFT {
    constructor(title, owner, token_id, url, isLocked,
                apr, borrowed_money, duration, real_owner, is_confirmed, creditor, start_time) {
        super(title, owner, token_id, url, isLocked);

        this.apr = apr;
        this.borrowed_money = borrowed_money;
        this.duration = duration;
        this.real_owner = real_owner;
        this.is_confirmed = is_confirmed;
        this.creditor = creditor;
        this.start_time = start_time;
    }
}


