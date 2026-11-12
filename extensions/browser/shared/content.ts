import { extractLinks, isTypingTarget } from './extract.js';

declare const chrome: {
    runtime: {
        sendMessage: (message: unknown) => void;
        onMessage: {
            addListener: (
                callback: (message: { type: string }, sender: unknown, sendResponse: (response?: unknown) => void) => void,
            ) => void;
        };
    };
};

const capture = () =>
    chrome.runtime.sendMessage({
        type: 'captured_links',
        payload: { pageUrl: location.href, pageTitle: document.title, links: extractLinks() },
    });

document.addEventListener('keydown', event => {
    if (event.key === '*' && !event.ctrlKey && !event.metaKey && !event.altKey && !isTypingTarget(event.target)) {
        event.preventDefault();
        capture();
    }
});

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
    if (message.type === 'extract_links') {
        capture();
        // Acknowledge so the background script can tell this page already has a content script.
        sendResponse({ ok: true });
    }
});
