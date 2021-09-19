import { configureStore } from '@reduxjs/toolkit'
import userReducer from '../data/UserSlice'
import articleReducer from '../data/ArticleSlice'
import tagReducer from '../data/TagSlice'
import linkReducer from '../data/LinkSlice'

export default configureStore({
  reducer: {
    user: userReducer, 
    article: articleReducer,
    tag: tagReducer, 
    link: linkReducer
  },
})