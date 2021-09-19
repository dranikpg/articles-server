import { TextField, FormControlLabel, Checkbox, Button, Grid, Typography, Paper, Box, Container } from '@material-ui/core';
import React, { useState } from 'react'
import { useSelector, useDispatch } from 'react-redux'

import {tryAuth} from '../data/UserSlice'

function Auth() {
    const attempt = useSelector((state) => state.user.attempt)
    const dispatch = useDispatch();

    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [register, setRegister] = useState(false);

    const submit = () => {
        dispatch(tryAuth({email, password, register}));
    }

    return <Container maxWidth="sm">
        <Grid container 
            direction="column"
            spacing={3}>
            <Grid item>
                <Grid container justify="center">
                    <Grid item>
                        <Typography variant="h3">Login</Typography>
                    </Grid>
                </Grid>
            </Grid>
            <Grid item>
            {attempt.message && <Typography color="error">{attempt.message}</Typography>}
            </Grid>
            <Grid item>
                <TextField variant="outlined" fullWidth label="email"
                    value={email} onChange={(e) => setEmail(e.target.value)}/>
            </Grid>
            <Grid item>
                <TextField variant="outlined" fullWidth type="password" label="password"
                    value={password} onChange={(e) => setPassword(e.target.value)}/>
            </Grid>
            <Grid item>
                <FormControlLabel 
                    control={
                        <Checkbox type="checkbox" checked={register} onChange={(e) => setRegister(e.target.checked)} />
                    }
                    label="New account"/>
            </Grid>
            <Grid item>
                <Button fullWidth variant="outlined" 
                    onClick={submit}>Submit</Button>
            </Grid>
        </Grid>
    </Container>;
}

export default Auth;

/*
return <form>
        <Container maxWidth="sm">
            <Paper >
                <Grid container direction="column" justify="center" spacing={3} style={{margin: "10px"}}>
                    <Grid item>
                        <Typography>Login</Typography>
                    </Grid>
                    {attempt.message && <Typography style={{color: 'red'}}>{attempt.message}</Typography>}
                    <Grid item>
                        <TextField fullWidth label="email"
                            value={email} onChange={(e) => setEmail(e.target.value)}/>
                    </Grid>
                    <Grid item>
                        <TextField fullWidth type="password" label="password"
                            value={password} onChange={(e) => setPassword(e.target.value)}/>
                    </Grid>
                    <Grid item>
                        <FormControlLabel 
                            control={
                                <Checkbox type="checkbox" checked={register} onChange={(e) => setRegister(e.target.checked)} />
                            }
                            label="register"/>
                    </Grid>
                    <Grid item>
                        <Button onClick={submit}>Submit</Button>
                    </Grid>
                </Grid>
            </Paper>
        </Container>
    </form>;
*/