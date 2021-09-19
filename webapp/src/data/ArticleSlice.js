import { createSlice, createAsyncThunk } from '@reduxjs/toolkit'
import { apiFetch } from './utils';

const initialState = {}

export const load = createAsyncThunk('artcile/load', async (query) => {
    const params = new URLSearchParams(query).toString();
    const response = await apiFetch("/article?"+params);
    if (response.status == 200) {
        return response.json();
    } else {
        return [];
    }
});

export const loadSingle = createAsyncThunk('article/loadSingle', async (id) => {
    const response = await apiFetch("/article/"+id)
    return response.json()
});

export const deleteSingle = createAsyncThunk('article/delete', async (id) => {
    const response = await apiFetch("/article/"+id, {
        method: 'DELETE'
    });
    return response.status;
});

export const saveSingle = createAsyncThunk('article/saveSingle', async (article) => {
    const response = await apiFetch("/article", {
        method: 'POST',
        body: JSON.stringify(article)
    });
    return response.json()
});

export const articleSlice = createSlice({
    name: 'article',
    initialState, 
    reducers: {
    },
    extraReducers: {
    }
})


export default articleSlice.reducer