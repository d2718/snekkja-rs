// snekkja (https://github.com/d2718/snekkja-rs) front end
"use strict";

const LAYOUT = {
    top: document.getElementById("top"),
    container: document.getElementById("container"),
    focus: document.getElementById("focus"),
    caption: document.getElementById("caption"),
    strip: document.getElementById("strip"),
    zlayer: document.getElementById("zoom-layer"),
    zimage: document.getElementById("zoom"),
};
const STATE = { strip_idx: 0, cur_idx: 0, };

function clamp(x, low, high) {
    if(x < low) { return low; }
    else if(x > high) { return high; }
    else { return x; }
}

function clear(elt) {
    while(elt.firstChild) {
        clear(elt.lastChild);
        elt.removeChild(elt.lastChild);
    }
}

function preload_images() {
    for(const uri of FILES) {
        const img = new Image();
        img.src = uri;
    }
}

function calculate_n_thumbs() {
    const avail = LAYOUT.strip.clientWidth;
    const padded_prev_px = THUMB_SIZE + 8;
    var n_prevs = Math.floor(avail/padded_prev_px);
    if(n_prevs % 2 == 0) { n_prevs = n_prevs - 1; }
    if(n_prevs < 1) { n_prevs = 1; }
    if(n_prevs > FILES.length) { n_prevs = FILES.length; }
    return n_prevs;
}

function set_image(div, uri) {
    const img = document.createElement("img");
    const size_str = `${THUMB_SIZE}px`;
    div.style.width = size_str;
    div.style.height = size_str;
    img.onload = function() {
        if(img.naturalHeight > img.naturalWidth) {
            img.style.width = "100%";
        } else {
            img.style.height = "100%";
        }
    }
    img.src = uri;
    div.appendChild(img);
}

function focus_image(idx) {
    const n = clamp(idx, 0, FILES.length-1);
    const n_prevs = calculate_n_thumbs();
    const focus_offset = Math.floor(n_prevs/2);
    const max_strip_idx = FILES.length - n_prevs;
    const strip_idx = clamp(n - focus_offset, 0, max_strip_idx);
    const max_i = strip_idx + n_prevs;
    const arrow_skip = clamp(n_prevs - 1, 1, n_prevs);

    clear(LAYOUT.strip);

    for(let i = strip_idx; i < max_i; i++) {
        const div = document.createElement("div");
        set_image(div, FILES[i]);
        if(i == n) {
            div.setAttribute("class", "focused");
        }
        div.onclick = () => { focus_image(i); };
        LAYOUT.strip.appendChild(div);
    }

    if (strip_idx > 0) {
        let div = LAYOUT.strip.querySelector("div:first-child");
        let img = document.createElement("img");
        img.setAttribute("class", "arrow");
        img.src = "prev.svg";
        div.appendChild(img);
        img.onclick = () => {
            focus_image(STATE.cur_idx - arrow_skip);
        }
    }
    if (strip_idx < max_strip_idx) {
        let div = LAYOUT.strip.querySelector("div:last-child");
        let img = document.createElement("img");
        img.setAttribute("class", "arrow");
        img.src = "next.svg";
        div.appendChild(img);
        img.onclick = () => {
            focus_image(STATE.cur_idx + arrow_skip);
        }
    }

    const uri = FILES[n];
    LAYOUT.focus.src = uri;
    const caption = CAPTIONS.get(uri);
    if(caption) {
        LAYOUT.caption.innerHTML = caption;
        LAYOUT.caption.style.display = "block";
    } else if(DEFAULT_CAPTION) {
        LAYOUT.caption.innerHTML = DEFAULT_CAPTION;
        LAYOUT.caption.style.display = "block";
    } else {
        LAYOUT.caption.style.display = "none";
    }

    STATE.strip_idx = strip_idx;
    STATE.cur_idx = n;
}

LAYOUT.container.addEventListener("click", () => {
    LAYOUT.zimage.src = LAYOUT.focus.src;
    LAYOUT.zlayer.style.zIndex = "3";
});
LAYOUT.zlayer.addEventListener("click", () => {
    LAYOUT.zimage.src = null;
    LAYOUT.zlayer.style.zIndex = "-1";
});

function main() {
    preload_images();

    if(TITLE) {
        const div = document.createElement("h1");
        div.id = "title";
        div.innerHTML = TITLE;
        LAYOUT.top.insertBefore(div, LAYOUT.container);
        document.title = TITLE;
    }

    focus_image(0);
}

window.onresize = () => { focus_image(STATE.cur_idx); };

if(document.readyState == "complete") {
    main();
} else {
    window.addEventListener("load", main);
}