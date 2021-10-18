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


function mintbaseCard(data, id) {
    const result = {type: 'mintbase', text: '<code>MINTBASE</code>\n\n'};
    if (!data) return {text: result.text + 'No data'};
    if (id !== undefined && id !== '') result.text += '<i>id: </i>' + id + '';
    if (data.media && data.media !== '') result.text += ', <a href="' + data.media + '">image</a> ';
    if (data.animation_url && data.animation_url !== '') result.text += ', <a href="' + data.animation_url + '">animation</a> ';
    if (data.youtube_url && data.youtube_url !== '') result.text += ', <a href="' + data.youtube_url + '">youtube</a> ';
    if (data.document && data.document !== '') result.text += ', <a href="' + data.document + '">document</a> ';
    if (data.title && data.title !== '') result.text += '<i>\nname: </i><b>' + data.title.trim().replace(/[\<\>]/g, '') + '</b>\n';
    if (data.category && data.category.trim() !== '') result.text += '<i>category: </i>' + data.category.trim().replace(/[\<\>]/g, '') + '\n';
    if (data.description && data.description.trim() !== '') result.text += '<i>' + data.description.trim().replace(/[\<\>]/g, '') + '</i>\n';
    if (data.store && data.store !== '') result.text += '<i>store: </i>https://www.mintbase.io/store/' + data.store + '\n';
    result.media = data.media || data.animation_url || data.youtube_url;
    return result
}
