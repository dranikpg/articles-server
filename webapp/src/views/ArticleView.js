import { Container, Divider, Typography, Button, Grid } from '@material-ui/core';
import { Link as RouterLink } from 'react-router-dom';
import React, { useEffect, useState } from 'react'
import { useDispatch } from 'react-redux';
import { useParams } from 'react-router';
import { loadSingle } from '../data/ArticleSlice';

import ReactMarkdown from 'react-markdown'
import rehypeHighlight from 'rehype-highlight'
import remarkGfm from 'remark-gfm'
import moment from 'moment';
import ArticleTags from '../components/ArticleTags';


function ArticleView() {
    const {id} = useParams();
    const dispatch = useDispatch();
    const [article, setArticle] = useState(null);

    useEffect(() => {
        dispatch(loadSingle(id))
            .then(({payload: article}) => {
                setArticle(article);
            })
    }, []);

    if (article == null) {
        return "Loading...";
    }

    return <Container maxWidth="md" style={{paddingTop: "10px"}}>
        <Typography variant="h5"> 
            {article.title} 
        </Typography>

        <Typography variant="caption"> 
            Updated {moment(article.updated_on, "YYYY-MM-DD").fromNow()}
        </Typography>
        <Typography variant="caption"> 
            Created {moment(article.created_on, "YYYY-MM-DD").fromNow()}
        </Typography>
    
        <Grid container
            justifyContent="space-between"
            >
            <Grid item>
                <ArticleTags tags={article.tags} />
            </Grid>
            <Grid item>
                <Button variant="outlined" size="small" component={RouterLink} to={"/edit/"+id}>
                    Edit
                </Button>
            </Grid>
        </Grid>

        <ReactMarkdown rehypePlugins={[rehypeHighlight]} 
                remarkPlugins={[remarkGfm]} children={article.content}/>
                

    </Container>;
}

export default ArticleView;