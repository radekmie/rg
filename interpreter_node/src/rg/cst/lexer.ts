import { Lexer } from 'chevrotain';

import { tokens } from './tokens';

export const lexer = new Lexer(tokens);
