import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'

import { RootStore } from './stores/rootStore'
import { StoreProvider } from './stores/StoreProvider'
import App from './App.tsx'
import './index.css'

const store = new RootStore()

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <StoreProvider store={store}>
      <BrowserRouter>
        <App />
      </BrowserRouter>
    </StoreProvider>
  </StrictMode>,
)
