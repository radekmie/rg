import { createRoot } from 'react-dom/client';

import * as wasm from '../wasm';
import { Application } from './components/Application';
import * as styles from './index.module.css';

wasm.initPromise.then(
  () => {
    const main = document.createElement('main');
    const root = createRoot(main);

    main.className = styles.main;
    root.render(<Application />);
    document.body.appendChild(main);
  },
  error => {
    console.error(error);
    alert('Something went wrong :(\nPlease check the console.');
  },
);
