export {NFT, LockedNFT}

class NFT {
    constructor(title, owner, token_id, url, extra, type) {
        this.title = title;
        this.owner = owner;
        this.token_id = token_id;
        this.url = url;
        this.extra = extra;
        this.type = type;
    }
}



class LockedNFT extends NFT {
    constructor(title, owner, token_id, url, extra, type,
                apr, borrowed_money, duration, real_owner, state, creditor, start_time) {
        super(title, owner, token_id, url, extra, type);

        this.apr = apr;
        this.borrowed_money = borrowed_money;
        this.duration = duration;
        this.real_owner = real_owner;
        this.state = state;
        this.creditor = creditor;
        this.start_time = start_time;
    }
}


