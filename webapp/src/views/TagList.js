import { List, ListItem, ListItemText, Container, ListItemAvatar, Avatar } from '@material-ui/core';
import React, { useEffect, useState } from 'react'
import { useDispatch, useSelector } from 'react-redux';
import { reloadTags } from '../data/TagSlice';

function TagList() {
    const dispatch = useDispatch();
    const tags = useSelector((state) => state.tag.list);
    
    useEffect(() => {
        dispatch(reloadTags());
    }, []);

    return <Container size="md">
        <List>
            {tags.map((t) => <ListItem>
                <ListItemAvatar>
                    <Avatar> {t.num_articles} </Avatar>
                </ListItemAvatar>
                <ListItemText primary={t.name} />
            </ListItem>)}
        </List>
    </Container>
}
export default TagList;