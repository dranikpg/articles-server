import { createSlice, createAsyncThunk } from '@reduxjs/toolkit'

const initialState = {
    email: null,
    authorized: null,
    attempt: {
        running: false,
        email: null,
        status: null,
        message: null,
    }
}

export const init = createAsyncThunk('user/init', async () => {
    const response = await fetch("/user");
    const email = localStorage.getItem("email")
    if (response.status === 200) { // authorized
        return {
            email,
            authorized: true
        }
    } else if(response.status === 401) { // unauthorized
        return {
            email,
            authorized: false
        }
    } else {
        return {
            email,
            authorized: undefined
        }
    }
});

export const tryAuth = createAsyncThunk('user/auth', async ({email, password, register}) => {
    const response = await fetch("/login?register="+register, {
        method: 'POST',
        body: JSON.stringify({
            email,
            password
        })
    });
    if (response.status === 200) {
        return {
            status: 'OK',
            email
        }
    } else {
        const json = await response.json();
        return {
            status: json.kind,
            message: json.message
        }
    }
});

export const userSlice = createSlice({
    name: 'user',
    initialState, 
    reducers: {
    },
    extraReducers: {
        [init.fulfilled]: (state, action) => {
            state.email = action.payload.email;
            state.authorized = action.payload.authorized;
        },
        [tryAuth.pending]: (state, action) => {
            state.attempt.running = true;
        },
        [tryAuth.fulfilled]: (state, action) => {
            const {status, email, message, kind} = action.payload;
            state.attempt.running = false;
            state.attempt.status = status;
            state.attempt.email = email;
            state.attempt.kind = kind;
            state.attempt.message = message;

            if (status === 'OK') {
                state.email = email;
                state.authorized = true;
                localStorage.setItem("email", email);
            } 
        },
        [tryAuth.rejected]: (state, action) => {
            state.attempt.running = false;
            state.attempt.status = "Failed";
        }
    }
})


export default userSlice.reducer