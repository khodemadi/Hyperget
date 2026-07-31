import type { BrowserMessage } from "./types.js";

declare const chrome: any;

const host = "io.github.hyper_get";

function send(message: BrowserMessage) {
    return new Promise((resolve, reject) =>
        chrome.runtime.sendNativeMessage(host, message, (reply: unknown) =>
            chrome.runtime.lastError
                ? reject(new Error(chrome.runtime.lastError.message))
                : resolve(reply)
        )
    );
}

function createMenus() {
    chrome.contextMenus.removeAll(() => {
        chrome.contextMenus.create({
            id: "single",
            title: "Download with Hyper Get",
            contexts: ["link"],
        });

        chrome.contextMenus.create({
            id: "all",
            title: "Download all links with Hyper Get",
            contexts: ["page"],
        });
    });
}

createMenus();
chrome.runtime.onInstalled.addListener(createMenus);
chrome.runtime.onStartup.addListener(createMenus);

    chrome.contextMenus.create({
        id: "all",
        title: "Download all links with Hyper Get",
        contexts: ["page"],
    });
});

chrome.contextMenus.onClicked.addListener((info: any, tab: any) => {
    if (info.menuItemId === "single" && info.linkUrl) {
        void send({
            type: "send_single_download",
            payload: {
                url: info.linkUrl,
                pageUrl: tab.url,
                referer: tab.url,
            },
        }).catch(() =>
            chrome.notifications.create({
                type: "basic",
                iconUrl: "icons/icon128.png",
                title: "Hyper Get unavailable",
                message: "The browser download was not cancelled.",
            })
        );
    }

    if (info.menuItemId === "all" && tab.id) {
        chrome.tabs.sendMessage(tab.id, {
            type: "extract_links",
        });
    }
});

chrome.runtime.onMessage.addListener((message: any) => {
    if (message.type === "captured_links") {
        void send({
            type: "send_page_links",
            payload: message.payload,
        }).catch(() => undefined);
    }
});

chrome.action.onClicked.addListener(() => {
    void send({
        type: "open_application",
    }).catch(() =>
        chrome.notifications.create({
            type: "basic",
            iconUrl: "icons/icon128.png",
            title: "Hyper Get unavailable",
            message: "Could not launch Hyper Get.",
        })
    );
});
