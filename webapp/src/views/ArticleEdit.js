import moment from 'moment';
import React from 'react'
import { connect } from 'react-redux'

import _ from 'lodash';

import ReactMarkdown from 'react-markdown'
import rehypeHighlight from 'rehype-highlight'
import remarkGfm from 'remark-gfm'

import SwipeableViews from 'react-swipeable-views';


import {loadSingle, saveSingle, deleteSingle} from '../data/ArticleSlice'
import { reloadTags } from '../data/TagSlice';
import ArticleTags from '../components/ArticleTags'

import { withRouter } from 'react-router';
import { Typography, AppBar, Grid, Paper, LinearProgress, TextField, Tabs, Tab, Divider, makeStyles, Chip, withStyles, Hidden, Box, Button } from '@material-ui/core';
import { TabPanel, TabContext, TabList, Autocomplete } from '@material-ui/lab';


const styles = {
    tabPanel: {
        padding: '20px 0px 0px 0px',
        minHeight: '70vh'
    },
    chip: {
        marginRight: '5px',
        marginBottom: '10px'
    },
    content: {
        padding: '0px 15px 0px 15px'
    },
    markdown: {
        maxWidth: '95vw',
        minHeight: '20vh'
    },
    basegrid: {
        paddingTop: "10px"
    }
};

class DelayedTextField extends React.Component {
    state = {
        content: ""
    }
    constructor(props) {
        super(props);
        this.firstPass = true;
        if (props.content) {
            this.firstPass = false;
            this.state.content = props.content;
        }
        this.delayedUpdate = _.debounce(() => {
            this.props.update(this.state.content);
        }, 1000);
    }
    componentDidUpdate(prevProps) {
        if (this.firstPass) {
            this.setState({content: this.props.content});
            this.firstPass = false;
        }
    }
    render() {
        const {label, minRows, multiline} = this.props;
        const {content} = this.state;
        return <TextField fullWidth variant="outlined" label={label}
            minRows={minRows} multiline={multiline}
            value={content} onChange={(e) => {
                this.setState({content: e.target.value});
                this.delayedUpdate();
            }}/>;
    }
}

class ArticlEdit extends React.Component {
    state = {
        article: {
            title: "",
            content: "",
            tags: []
        },
        dirty: false,
        loading: true,
        tabIndex: 0
    }

    constructor(props) {
        super(props);
        this.setArticleField = (field, value) => {
            this.setState({
                ...this.state, 
                dirty: true,
                article: {
                    ...this.state.article, 
                    [field]: value
                }
            });
        }

        this.saveArticle = _.debounce(() => {
            this.props.saveSingle(this.state.article)
                .then((action) => {
                    if (typeof action.payload === "number") {
                        this.state.article.id = action.payload;
                    }
                    this.setState({...this.state, dirty: false});
                });
        }, 2000);
    }

    componentDidUpdate(prevProps) {
        let prevId = prevProps.match.params.id;
        let curId = this.props.match.params.id;
        if (prevId != curId) {
            this.loadArticle(curId);
        }
    }

    componentDidMount() {
        let curId = this.props.match.params.id;
        this.loadArticle(curId);
        this.props.reloadTags();
    }

    loadArticle(id) {
        if (id == undefined) {
            this.setState({
                article: {
                    id: undefined,
                    title: "New article p2",
                    content: "Some content",
                    tags: []
                },
                loading: false,
                dirty: true,
            });
        } else {
            this.props.loadSingle(id)
                .then((action) => {
                    this.setState({
                        ...this.state, 
                        loading: false,
                        article: action.payload
                    })
                });
        }
        
    }

    deleteArticle() {
        if (window.confirm("Do you really want to delete the article?")) {
            this.props.deleteSingle(this.state.article.id);
            this.props.history.replace("/");
        }
    }
    
    renderEditPanel() {
        const {liveContent, article} = this.state;
        const {tags} = this.props;
        const setLiveContent = (liveContent) => {
            this.setArticleField("content", liveContent);
            this.saveArticle();
        }
        const setLiveTitile = (liveTitle) => {
            this.setArticleField("title", liveTitle);
            this.saveArticle();
        }
        const setLiveTags = (liveTags) => {
            this.setArticleField("tags", liveTags);
            this.saveArticle();
        }
        return <Box>
                <TextField variant="outlined" label="title"
                    fullWidth 
                    value={article.title} onChange={(e) => setLiveTitile(e.target.value)}/>
                <DelayedTextField multiline label="content"
                    minRows={10} content={article.content}
                    update={(c) => setLiveContent(c)} />
                <Autocomplete
                    multiple freeSolo
                    value={article.tags}
                    onChange={(event, newTags) => setLiveTags(newTags)}
                    options={tags}
                    getOptionLabel={(option) => option}
                    renderInput={(params) => (
                        <TextField
                          {...params}
                          variant="standard"
                          label="Tags"
                        />
                      )}
                />
                <Grid container
                    style={{paddingTop: '10px'}}
                    alignItems="center"
                    justifyContent="space-between">
                    <Grid item>
                        {article.id &&
                            <Button color="secondary" size="small"
                                onClick={() => this.deleteArticle()}> 
                                Delete 
                            </Button>
                        }
                    </Grid>
                    <Grid item>
                        {this.state.dirty ? 
                        <Chip label="Editing" variant="outlined" 
                                color="secondary" size="small"/>
                        : <Chip label="Saved" variant="outlined" 
                                color="primary" size="small"/>}
                    </Grid>
                </Grid>
            </Box>
    }
    
    renderViewPanel() {
        const {article} = this.state;
        const {classes} = this.props;

        return <Box className={classes.content}>
            <Typography variant="h5">{article.title}</Typography>
            <Typography className={classes.created_label} variant="caption" display="inline">
                {article.created_on ? 
                    moment(article.created_on, "YYYY-MM-DD").fromNow()
                    : "today"}
            </Typography>
            <Divider style={{marginBottom: '10px'}}/>
            <ArticleTags tags={article.tags}/>
            <ReactMarkdown rehypePlugins={[rehypeHighlight]} 
                remarkPlugins={[remarkGfm]} children={article.content} 
                className={classes.markdown}/>
        </Box>;
    }

    rendeMobile() {
        const {classes} = this.props;
        return <TabContext value={this.state.tabIndex}>
            <Grid container
                direction="row" justifyContent="space-evenly">
                    
                <Grid item xs={12}>
                    <AppBar position="static">
                        <TabList variant="fullWidth" 
                        onChange={(e, val) => this.setState({...this.state, tabIndex: val})}>
                            <Tab label="View" value={0}/>
                            <Tab label="Edit" value={1}/>
                        </TabList>
                    </AppBar>
                </Grid>
                <Grid item xs={12} style={{paddingBottom: "10px"}}>
                    <SwipeableViews
                        index={this.state.tabIndex}
                        onChangeIndex={(idx) => this.setState({...this.state, tabIndex: idx})}>
                        <TabPanel value={0} className={classes.tabPanel}>
                            {this.renderViewPanel()}
                        </TabPanel>
                        <TabPanel value={1} className={classes.tabPanel}>
                            {this.renderEditPanel()}
                        </TabPanel>
                    </SwipeableViews>
                </Grid>
            </Grid>
        </TabContext>
    }

    renderDesktop() {
        const {classes} = this.props;
        if (this.state.loading) {
            return <LinearProgress />
        }
        return <Grid container className={classes.basegrid}
            direction="row" justifyContent="space-evenly"
            spacing={3}>
            <Grid item md={6}>
                {this.renderEditPanel()}
            </Grid>
            <Grid item md={6}>
                {this.renderViewPanel()}
            </Grid>
        </Grid>
    }

    render() {
        return <React.Fragment>
            <Hidden mdUp>
                {this.rendeMobile()}
            </Hidden>

            <Hidden smDown>
                {this.renderDesktop()}
            </Hidden>
        </React.Fragment>
    }
}

const mapStateToProps = (state) => {
    return {
        tags: state.tag.list.map(t => t.name)
    }
}

export default connect(mapStateToProps, {loadSingle, saveSingle, deleteSingle, reloadTags})(withRouter(withStyles(styles)(ArticlEdit)));