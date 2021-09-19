import React from 'react'
import { Chip } from '@material-ui/core';
import { makeStyles, createStyles } from '@material-ui/core/styles';

const useStyles = makeStyles((theme) => createStyles({
    chip: {
      marginRight: '5px',
      marginTop: '2px'
    },
}));

function ArticleTags({tags}) {
    const classes = useStyles();
    if (!tags) {
        return ""
    }
    return tags.map(
        tag => <Chip className={classes.chip} variant="outline" size="small" label={tag}/>)
}

export default ArticleTags;