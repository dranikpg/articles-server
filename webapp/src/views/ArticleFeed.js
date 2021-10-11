import { Container, Link, Button, Card, Grid, CardContent, LinearProgress, Typography, TextField, Chip, Box, Divider, Switch, FormControlLabel } from '@material-ui/core';
import { makeStyles } from '@material-ui/core/styles';
import { Link as RouterLink } from 'react-router-dom';
import { Autocomplete } from '@material-ui/lab';
import moment from 'moment';
import React, { useEffect, useState, useRef, useCallback } from 'react'
import { useSelector, useDispatch } from 'react-redux'
import ArticleTags from '../components/ArticleTags';
import InfiniteScroll from 'react-infinite-scroll-component';
import {debounce, initial} from 'lodash';

import {load as loadArticles} from '../data/ArticleSlice'
import { reloadTags } from '../data/TagSlice';

const useStyles = makeStyles((theme) => ({
  chip: {
    marginLeft: '5px',
  },
  viewContainer: {
    paddingTop: '10px'
  }
}));
  
function Article(props) {
    const {article} = props;
    const classes = useStyles();
    let ts = [];
    for (const [idx, part] of article.preview.split("**").entries()) {
        let style = {};
        if (idx % 2 == 1) {
            style.textDecoration = "underline blue";
            style.fontWeight = "bold";
        }
        ts.push(<span style={style}>{part}</span>)
    }
    return <React.Fragment>
        <Typography gutterBottom variant="h6" component="h2">
            <Link component={RouterLink} to={"/view/"+article.id}>
                {article.title}
            </Link>
            <span style={{paddingLeft: 5}}/>
            <ArticleTags tags={article.tags} />
        </Typography>
        <p>
            {ts}
        </p>
        { article.created_on != article.updated_on && <Typography className={classes.createdLabel} variant="caption" display="inline">
            updated {moment.utc(article.updated_on).fromNow()}
        </Typography> }
        <br/>
        <Typography className={classes.createdLabel} variant="caption" display="inline">
            created {moment.utc(article.created_on).fromNow()}
        </Typography> 
    </React.Fragment>;
}

function ArticleFeed() {
    const dispatch = useDispatch()
    const tags = useSelector((state) => state.tag.list)
    const isInitial = useRef(true)
    const [articles, setArticles] = useState({
        list: [],
        hasNext: true
    });
    const [filter, setFilter] = useState({
        tags: [],
        allTags: false,
        query: "",
        sortBy: "created"
    });
    const [rawQuery, setRawQuery] = useState("");
    const debouncedQueryFilterUpdate = useRef(debounce((rq) => {
        setFilter({
            ...filter,
            query: rq
        });
    }, 500));

    const loadMore = (fresh = false) => {
        let tagids = filter.tags.length == 0 ? undefined : filter.tags.map(t => t.id);
        let query = {
            from: fresh ? 0 : articles.list.length,
            limit: 10,
            all_tags: filter.allTags,
            sort_by: filter.sortBy
        };
        if (tagids && tagids.length > 0) {
            query.tags = tagids;
        }
        if (filter.query) {
            query.query = filter.query;
        }
        dispatch(loadArticles(query))
            .then(({payload: moreArticles}) => {
                const fullList = (fresh ? [] : articles.list).concat(moreArticles)
                setArticles({
                    list: fullList,
                    hasNext: (moreArticles && moreArticles.length == 10)
                })
            })
    }

    useEffect(() => {
        dispatch(reloadTags());
    }, [dispatch]);

    useEffect(() => {
        setArticles({
            list: [],
            hasNext: true
        });
        if (isInitial.current) {
            isInitial.current = false;
            return;
        }
        loadMore(true);
    }, [filter]);

    useEffect(() => {
        debouncedQueryFilterUpdate.current(rawQuery);
    }, [rawQuery]);

    const views = articles.list.map((a) => <Card variant="outlined" style={{marginTop: "10px"}}>
        <CardContent>
            <Article article={a} />
        </CardContent>
    </Card>);

    return <Container maxWidth="md" style={{marginTop: 10}}>
        <Grid container
            spacing={2}
            style={{padding: '5px'}}>
            <Grid item>
                <Button variant="outlined" size="small" component={RouterLink} to={"/create"}>
                    Create New
                </Button>
            </Grid>
            <Grid item>
                <Button variant="outlined" size="small" component={RouterLink}  to="/tags">
                    View tags
                </Button>
            </Grid>
            <Grid item>
                <Button variant="outlined" size="small" component={RouterLink}  to="/links">
                    View links
                </Button>
            </Grid>
        </Grid>
        <TextField
            fullWidth
            value={rawQuery}
            onChange={(e) => setRawQuery(e.target.value)}
            label="Full text search"
        />
        <Grid container
            spacing={2}
            alignItems="flex-end">
            <Grid item xs={10}>
                <Autocomplete
                    multiple
                    value={filter.tags}
                    onChange={(event, newTags) => {
                        setFilter({
                            ...filter,
                            tags: newTags
                        });
                    }}
                    options={tags}
                    getOptionLabel={(option) => option.name}
                    renderInput={(params) => (
                        <TextField
                          {...params}
                          variant="standard"
                          label="Filter by tags"
                          placeholder="tag"
                        />
                      )}
                />
            </Grid>
            <Grid item xs={1}>
                <FormControlLabel control={
                    <Switch size="small" 
                        checked = {filter.allTags}
                        onChange = {(e) => setFilter({...filter, allTags: e.target.checked})}
                    />} 
                label="&&"/>
            </Grid>
            <Grid item xs={1}>
                <Button variant="outlined" size="small"
                    onClick={() => setFilter({...filter, sortBy: filter.sortBy == "created" ? "updated" : "created"})}
                    >
                    {filter.sortBy}
                </Button>
            </Grid>
        </Grid>

        <InfiniteScroll
            dataLength={articles.list.length} //This is important field to render the next data
            next={loadMore}
            hasMore={articles.hasNext}
            loader={<LinearProgress />}
        >
            {views}
        </InfiniteScroll>
    </Container>;
}

export default ArticleFeed;