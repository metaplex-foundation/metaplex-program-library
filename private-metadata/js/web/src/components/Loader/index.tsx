import React from 'react';

export const LoadingContext = React.createContext({});

export const LoaderProvider: React.FC = ({ children }) => {
  const [loading, setLoading] = React.useState(false);
  return (
    <LoadingContext.Provider
      value={{
        loading,
        setLoading,
      }}
    >
      <div className={`loader-container ${loading ? 'active' : ''}`}>
        <div className="loader-block">
          <div className="loader-title">loading</div>
          <Spinner />
        </div>
      </div>
      {children}
    </LoadingContext.Provider>
  );
};

export const useLoading = (): any => {
  const context = React.useContext(LoadingContext);
  if (!context) {
    throw new Error(`useLoading must be used with a LoadingProvider`);
  }
  return context;
};

export const Spinner = () => {
  return (
    <div className="spinner">
      <span className="line line-1" />
      <span className="line line-2" />
      <span className="line line-3" />
      <span className="line line-4" />
      <span className="line line-5" />
      <span className="line line-6" />
      <span className="line line-7" />
      <span className="line line-8" />
      <span className="line line-9" />
    </div>
  );
};
