import fetchAbsolute from "fetch-absolute";

const apiFetch = fetchAbsolute(fetch)(process.env.PUBLIC_URL+"/api");

export {apiFetch};