import { render } from 'solid-js/web';
import { Router } from '@solidjs/router';
import './index.css';
import App from './App';

const root = document.getElementById('root');

if ((import.meta as { env?: { DEV?: boolean } }).env?.DEV && !(root instanceof HTMLElement)) {
  throw new Error(
    'Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?',
  );
}

render(() => (
  <Router>
    <App />
  </Router>
), root!);