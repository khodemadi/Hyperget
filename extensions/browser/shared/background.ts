import type { BrowserMessage } from "./types.js";

declare const chrome: any;

const host = "io.github.hyper_get";

// Chromium MV3 exposes `action`; Firefox MV2 only has `browserAction`.
const toolbarAction = chrome.action ?? chrome.browserAction;

const ignoreLastError = (): void => {
    void chrome.runtime.lastError;
};

function send(message: BrowserMessage): Promise<unknown> {
    return new Promise((resolve, reject) => {
        chrome.runtime.sendNativeMessage(host, message, (reply: any) => {
            if (chrome.runtime.lastError) {
                reject(new Error(chrome.runtime.lastError.message));
            } else if (!reply || reply.accepted === false) {
                reject(new Error(reply?.error ?? "Native host rejected the message"));
            } else {
                resolve(reply);
            }
        });
    });
}

function notify(title: string, message: string): void {
    chrome.notifications.create({
        type: "basic",
        iconUrl: "icons/icon128.png",
        title,
        message,
    });
}

function createMenus(): void {
    chrome.contextMenus.removeAll(() => {
        ignoreLastError();

        chrome.contextMenus.create(
            {
                id: "single",
                title: "Download with Hyper Get",
                contexts: ["link"],
            },
            ignoreLastError,
        );

        chrome.contextMenus.create(
            {
                id: "all",
                title: "Download all links with Hyper Get",
                contexts: ["page"],
            },
            ignoreLastError,
        );
    });
}

createMenus();

chrome.runtime.onInstalled.addListener(createMenus);
chrome.runtime.onStartup.addListener(createMenus);

// Pages opened before the extension was installed/reloaded have no content
// script yet, so inject it on demand.
function injectContentScript(tabId: number): Promise<void> {
    return new Promise((resolve, reject) => {
        const done = () => {
            if (chrome.runtime.lastError) {
                reject(new Error(chrome.runtime.lastError.message));
            } else {
                resolve();
            }
        };

        if (chrome.scripting?.executeScript) {
            chrome.scripting.executeScript({ target: { tabId }, files: ["content.js"] }, done);
        } else {
            chrome.tabs.executeScript(tabId, { file: "content.js" }, done);
        }
    });
}

function requestLinkExtraction(tabId: number): void {
    const ask = () =>
        new Promise<void>((resolve, reject) => {
            chrome.tabs.sendMessage(tabId, { type: "extract_links" }, () => {
                if (chrome.runtime.lastError) {
                    reject(new Error(chrome.runtime.lastError.message));
                } else {
                    resolve();
                }
            });
        });

    ask()
        .catch(async () => {
            await injectContentScript(tabId);
            await ask();
        })
        .catch(() => {
            notify("Hyper Get", "Could not read links from this page.");
        });
}

chrome.contextMenus.onClicked.addListener((info: any, tab: any) => {
    if (info.menuItemId === "single" && info.linkUrl) {
        void send({
            type: "send_single_download",
            payload: {
                url: info.linkUrl,
                pageUrl: tab?.url,
                referer: tab?.url,
            },
        }).catch(() => {
            notify("Hyper Get unavailable", "The browser download was not cancelled.");
        });

        return;
    }

    if (info.menuItemId === "all" && tab?.id !== undefined) {
        requestLinkExtraction(tab.id);
    }
});

chrome.runtime.onMessage.addListener((message: any) => {
    if (message?.type === "captured_links") {
        void send({
            type: "send_page_links",
            payload: message.payload,
        }).catch(() => {
            notify("Hyper Get unavailable", "Could not send the page links.");
        });
    }
});

toolbarAction.onClicked.addListener(() => {
    void send({
        type: "open_application",
    }).catch(() => {
        notify("Hyper Get unavailable", "Could not launch Hyper Get.");
    });
});
