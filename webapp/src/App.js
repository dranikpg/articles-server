import React from 'react';
import { connect } from 'react-redux'

import {init as initUser} from './data/UserSlice'

import Auth from './views/Auth';
import ArticleFeed from './views/ArticleFeed';
import ArticlEdit from './views/ArticleEdit';
import { Route, Switch } from 'react-router';
import ArticleView from './views/ArticleView';
import TagList from './views/TagList';
import LinkList from './views/LinkList';


class App extends React.Component {
    componentDidMount = () => {
        this.props.initUser()
    }
    renderBase = () => {
        return <Switch>
            <Route path="/view/:id">
                <ArticleView />
            </Route>
            <Route path="/edit/:id">
                <ArticlEdit />
            </Route>
            <Route path="/create">
                <ArticlEdit />
            </Route>
            <Route path="/tags">
                <TagList />
            </Route>
            <Route path="/links">
                <LinkList />
            </Route>
            <Route path="/">
                <ArticleFeed />
            </Route>
        </Switch>
    }
    render = () => {
        if (this.props.authorized === true) {
            return this.renderBase();
        } else if (this.props.authorized === false) {
            return <Auth />
        } else {
            return "Loading";
        }
    }
}

const mapStateToProps = (state) => state.user

export default connect(mapStateToProps, { initUser })(App)