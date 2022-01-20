import React from 'react'
import ReactDOM from 'react-dom'
import './index.css'
import App from './App'
import {BrowserRouter, Route, Routes} from "react-router-dom";
import AuthRedirect from "./AuthRedirect";
import AuthCallback from "./AuthCallback";

ReactDOM.render(
  <React.StrictMode>
      <BrowserRouter>
          <Routes>
              <Route path="/" element={<App />} />
              <Route path="/lg" element={<AuthRedirect />} />
              <Route path="/callback" element={<AuthCallback />} />
          </Routes>
      </BrowserRouter>
  </React.StrictMode>,
  document.getElementById('root')
)
