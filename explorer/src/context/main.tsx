import { PaletteMode } from '@mui/material';
import * as React from 'react';
import { GatewayResponse, MixNodeResponse, ValidatorsResponse } from 'src/typeDefs/node-status-api-client';
import { Api } from '../api';

type NodeApiResponse = {
  data: MixNodeResponse | GatewayResponse | ValidatorsResponse | number | null
  error: string | null
}
interface State {
  mode: PaletteMode
  toggleMode: () => void
  mixnodes: NodeApiResponse | null
  gateways: NodeApiResponse | null
  validators: NodeApiResponse | null
  block: NodeApiResponse | null
};

export const MainContext = React.createContext({} as State);

export const MainContextProvider: React.FC = ({ children }: any) => {
  // light/dark mode
  const [mode, setMode] = React.useState<PaletteMode>('light');

  // various APIs for cards on Overview
  const [mixnodes, setMixnodes] = React.useState<NodeApiResponse | null>(null);
  const [gateways, setGateways] = React.useState<NodeApiResponse | null>(null);
  const [validators, setValidators] = React.useState<NodeApiResponse | null>(null);
  const [block, setBlock] = React.useState<NodeApiResponse | null>(null);

  const toggleMode = () => setMode((m) => (m !== 'light' ? 'light' : 'dark'));

  const fetchMixnodes = async () => {
    try {
      const res = await Api.fetchMixnodes();
      setMixnodes({ data: res, error: null });
    } catch (error: any) {
      setMixnodes({ data: null, error: error.message });
    }
  };

  const fetchGateways = async () => {
    try {
      const res = await Api.fetchGateways();
      setGateways({ data: res, error: null });
    } catch (error:any) {
      setGateways({ data: null, error: error.message });
    }
  };

  const fetchValidators = async () => {
    try {
      const res = await Api.fetchValidators();
      setValidators({ data: res, error: null });
    } catch (error:any) {
      setValidators({ data: null, error: error.message });
    }
  };
  const fetchBlock = async () => {
    try {
      const res = await Api.fetchBlock();
      setBlock({ data: res, error: null });
    } catch (error:any) {
      setBlock({ data: null, error: error.message });
    }
  };

  React.useEffect(() => {
    Promise.all([
      fetchMixnodes(),
      fetchGateways(),
      fetchValidators(),
      fetchBlock(),
    ])
  }, []);

  return (
    <MainContext.Provider
      value={{ mode, toggleMode, mixnodes, gateways, validators, block }}
    >
      {children}
    </MainContext.Provider>
  );
};
