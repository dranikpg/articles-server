import { createSlice, createAsyncThunk } from '@reduxjs/toolkit'
import { apiFetch } from './utils';

const initialState = {
    list: []
}

export const load = createAsyncThunk('link/reload', async () => {
    const response = await apiFetch("/links");
    if (response.status === 200) {
        const json = await response.json();
        return json;
    } else {
        return [];
    }
});

export const linkSlice = createSlice({
    name: 'link',
    initialState, 
    reducers: {
    },
    extraReducers: {
        [load.fulfilled]: (state, action) => {
            state.list = action.payload;
        }
    }
})


export default linkSlice.reducer