import { List, ListItem, ListItemText, Container, Link, Divider, Typography } from '@material-ui/core';
import React, { useEffect, useState } from 'react'
import { useDispatch, useSelector } from 'react-redux';
import { load as loadLinks } from '../data/LinkSlice';
import { Link as RouterLink } from 'react-router-dom';

function LinkList() {
    const dispatch = useDispatch();
    const links = useSelector((state) => state.link.list);
    
    useEffect(() => {
        dispatch(loadLinks());
    }, []);

    return <Container size="md">
        <List>
            {links.map((t) => <React.Fragment>
                <ListItem>
                    <ListItemText primary={t.title} secondary={
                        <React.Fragment>
                            <Link href={t.url}> {t.url} </Link>
                            <br/>
                            from <Link to={"/view/"+t.article_id} component={RouterLink}> {t.article_title} </Link>
                        </React.Fragment>
                    } />
                </ListItem>
                <Divider component="li" />
            </React.Fragment>)}
        </List>
    </Container>
}
export default LinkList;