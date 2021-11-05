module.exports = {getMintbase, mintbaseCard};

const fetch = require('node-fetch');


async function getMintbase(url) {
    try {
        const res = await fetch(url, {timeout: 10000});
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


function mintbaseCard(data, is_uri) {
    console.log(data, is_uri);
    if (is_uri){
        return  {
            type: 'mintbase',
            media: (data.media || data.animation_url || data.youtube_url),
            title: (data.title || "-")
        };
    }
    return {
        ref: data.reference
    }
}
