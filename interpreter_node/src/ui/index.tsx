import { createRoot } from 'react-dom/client';

import { Application } from './components/Application';
import * as styles from './components/Application.module.css';

const main = document.createElement('main');
const root = createRoot(main);

main.className = styles.main;
root.render(<Application />);
document.body.appendChild(main);
