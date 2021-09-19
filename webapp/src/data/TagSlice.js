import { createSlice, createAsyncThunk } from '@reduxjs/toolkit'

const initialState = {
    list: []
}

export const reloadTags = createAsyncThunk('tag/reload', async () => {
    const response = await fetch("/tags");
    if (response.status === 200) {
        const json = await response.json();
        return json;
    } else {
        return [];
    }
});

export const tagSlice = createSlice({
    name: 'tag',
    initialState, 
    reducers: {
    },
    extraReducers: {
        [reloadTags.fulfilled]: (state, action) => {
            state.list = action.payload;
        }
    }
})


export default tagSlice.reducer