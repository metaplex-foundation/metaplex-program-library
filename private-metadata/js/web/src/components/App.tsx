import * as React from "react";
import { hot } from "react-hot-loader";

export const App = () => {
  return (
    <div className="app">
      <h1>Hello World!</h1>
    </div>
  );
}

declare let module: Record<string, unknown>;

export default hot(module)(App);
