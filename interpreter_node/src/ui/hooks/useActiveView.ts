import { useReducer } from 'react';

export enum View {
  AST,
  Automaton,
  Bench,
  CST,
  Graphviz,
  HighLevel,
  IST,
  LowLevel,
}

function reducer(_: View, view: View) {
  return view;
}

export function useActiveView() {
  const [activeView, setActiveView] = useReducer(reducer, View.Automaton);
  return { activeView, setActiveView };
}
