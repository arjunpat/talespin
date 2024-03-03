import { browser } from '$app/environment';
import { writable } from 'svelte/store';

export const nameStore = writable(browser ? window.localStorage.getItem("name") || "" : "");

nameStore.subscribe(value => {
    if (browser) {
        window.localStorage.setItem("name", value);
    }
});
